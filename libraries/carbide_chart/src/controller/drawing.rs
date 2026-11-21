//! Drawing shared by the line and scatter controllers, which differ only in which parts they draw
//! by default: a line chart draws the line through each dataset's points and the points; a scatter
//! chart draws the points, and the line only with `show_line`. Where a line is drawn, so is the
//! area under it, for datasets that `fill`.

use carbide::color::Color;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use carbide::draw::{Position, Rect, Scalar};
use carbide::draw::color::{rgba, Rgba};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataSetOptions, Fill, SpanGaps};
use crate::mark::{AsDataSet, LegendContext};
use crate::sequence::Sequence;
use crate::controller::line_path::{trace_line, trace_line_reversed, PathSink};
use crate::controller::orientation::{normalized_pixel, pixel, Oriented, Orientation};
use crate::controller::stacking::{StackKey, StackSums};
use crate::scale::{Axis, Scale, ScaleBounds, Scales};
use crate::legend::{LegendItem, LegendMarker};
use crate::style::series_style;
use carbide::environment::Environment;

/// Reports every dataset's x and y range to the scales it is bound to: the x range to its index
/// scale and the y range to its value scale, as `orientation` picks them (see
/// `Orientation::index_and_value_scales`). With `fills`, when the controller draws lines, a filled
/// dataset's baseline value is included too, so the area's base is on the chart; a `Fill::Stack`
/// area with a line below it fills to that line, so it includes nothing. With `stacks`, a dataset
/// in a stack reports the running totals it is drawn at instead of its own values. Empty datasets
/// have no bounds and contribute nothing, and neither do the datasets `counted` turns down, whose
/// points the controller doesn't draw.
pub(crate) fn collect_point_bounds<T, D>(datasets: &D, bounds: &mut ScaleBounds, fills: bool, stacks: bool, orientation: Orientation, counted: impl Fn(usize) -> bool)
where
    T: AsDataSet + ?Sized,
    D: Sequence<T>,
{
    let mut sums = StackSums::single();
    // The stacks that already have a non-empty line, which a later `Fill::Stack` fills down to.
    let mut filled_stacks: HashSet<StackKey> = HashSet::new();

    datasets.foreach(&mut |dataset_index, dataset| {
        if !counted(dataset_index) {
            return;
        }

        let dataset = dataset.as_dataset();
        let (Some((x_min, y_min)), Some((x_max, y_max))) = (dataset.min(), dataset.max()) else {
            return;
        };

        let options = dataset.options();
        let (index_scale, value_scale) = orientation.index_and_value_scales(&options);
        let key = StackKey::new(&options, dataset_index);
        let has_line_below = stacks && matches!(key, StackKey::Stacked { .. }) && !filled_stacks.insert(key.clone());

        bounds.include(index_scale, x_min, x_max);

        if stacks {
            dataset.points(&mut |_index, point| {
                let (Some(x), Some(y)) = (point.x(), point.y()) else {
                    return;
                };

                let (_, total) = sums.add(&key, x, y);
                bounds.include(value_scale, total, total);
            });
        } else {
            bounds.include(value_scale, y_min, y_max);
        }

        let fills_to_line = options.fill == Fill::Stack && has_line_below;

        if fills && !fills_to_line && let Some(baseline) = options.fill.baseline_value() {
            bounds.include(value_scale, baseline, baseline);
        }
    });
}

/// One dataset's line as it is drawn: its points that aren't missing, in pixels in the index frame
/// (see `Orientation`), at their running totals when it is stacked; the runs it is drawn in
/// without a break (see `SpanGaps`); and the line below it in its stack, which a `Fill::Stack`
/// area fills down to.
pub(crate) struct DrawnLine {
    dataset_index: usize,
    options: DataSetOptions,
    orientation: Orientation,
    /// In the index frame: x along the categories and y along the values.
    points: Vec<Position>,
    colors: Vec<DataColor>,
    /// Each drawn point's index in its dataset, which colours by point go by.
    point_indices: Vec<usize>,
    /// The unbroken runs of the line, as ranges of `points`.
    runs: Vec<Range<usize>>,
    /// The index, in the controller's lines, of the line below this one in its stack.
    below: Option<usize>,
}

/// Splits a line into the runs it is drawn in without a break. `points` holds, for each of the
/// dataset's points in order, its x value, or `None` when it is missing; the runs are ranges over
/// the points that aren't missing. `SpanGaps::Never` breaks at every missing point; `Always`
/// never breaks; `Within(max)` breaks between two neighbouring points further apart along x than
/// `max`, whether or not missing points lie between them.
pub(crate) fn split_runs(points: &[Option<Scalar>], span_gaps: SpanGaps) -> Vec<Range<usize>> {
    let mut runs = vec![];
    let mut start = 0;
    let mut drawn = 0;
    let mut previous_x: Option<Scalar> = None;

    for point in points {
        match point {
            None => {
                if span_gaps == SpanGaps::Never && drawn > start {
                    runs.push(start..drawn);
                    start = drawn;
                }
            }
            Some(x) => {
                if let (SpanGaps::Within(max), Some(previous)) = (span_gaps, previous_x)
                    && (x - previous).abs() > max && drawn > start {
                        runs.push(start..drawn);
                        start = drawn;
                    }

                drawn += 1;
                previous_x = Some(*x);
            }
        }
    }

    if drawn > start {
        runs.push(start..drawn);
    }

    runs
}

/// The lines of a controller's datasets, in dataset order, mapped through their scales into
/// `chart_area`, each point's x along its index scale and its y along its value scale, as
/// `orientation` picks them. With `stacks`, datasets in the same stack are drawn at their running
/// totals, one sum per stack (see `StackSums::single`), and each knows the line below it.
/// Datasets bound to a scale the chart doesn't have are left out, and so are the datasets
/// `counted` turns down, whose lines a mark replaces; they don't stack either.
pub(crate) fn drawn_lines<T, D>(datasets: &D, scales: &Scales, chart_area: Rect, stacks: bool, orientation: Orientation, counted: impl Fn(usize) -> bool) -> Vec<DrawnLine>
where
    T: AsDataSet + ?Sized,
    D: Sequence<T>,
{
    let mut lines: Vec<DrawnLine> = vec![];
    let mut sums = StackSums::single();
    let mut tops: HashMap<StackKey, usize> = HashMap::new();

    datasets.foreach(&mut |dataset_index, dataset| {
        if !counted(dataset_index) {
            return;
        }

        let dataset = dataset.as_dataset();
        let options = dataset.options();

        let (index_id, value_id) = orientation.index_and_value_scales(&options);
        let (Some(index_scale), Some(value_scale)) = (scales.get(index_id), scales.get(value_id)) else {
            return;
        };

        let key = StackKey::new(&options, dataset_index);
        let mut points = vec![];
        let mut colors = vec![];
        let mut point_indices = vec![];
        let mut xs = vec![];

        dataset.points(&mut |point_index, point| {
            // A missing point isn't drawn, adds nothing to its stack, and may break the line.
            let (Some(x), Some(y)) = (point.x(), point.y()) else {
                xs.push(None);
                return;
            };

            let y = if stacks { sums.add(&key, x, y).1 } else { y };

            points.push(Position::new(
                pixel(orientation.index_axis(), index_scale, x, chart_area),
                pixel(orientation.value_axis(), value_scale, y, chart_area),
            ));
            colors.push(point.color());
            point_indices.push(point_index);
            xs.push(Some(x));
        });

        let runs = split_runs(&xs, options.span_gaps);

        let below = if stacks && matches!(key, StackKey::Stacked { .. }) { tops.get(&key).copied() } else { None };

        // An empty line can't be filled down to, so the next one fills to the line below it.
        if stacks && !points.is_empty() {
            tops.insert(key, lines.len());
        }

        lines.push(DrawnLine { dataset_index, options, orientation, points, colors, point_indices, runs, below });
    });

    lines
}

/// Fills the area between each run of each filled line and its `Fill` baseline, in the dataset's
/// colour, at the `SeriesStyle`'s fill opacity. The area's top follows the run exactly, through
/// `trace_line`, then closes along the baseline, or, for `Fill::Stack`, back along the line below
/// it in its stack, traced in reverse with that line's own interpolation, so the area fits
/// exactly between the two drawn lines. A run needs two points to have an area.
pub(crate) fn draw_fills(lines: &[DrawnLine], ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect, palette: &Palette) {
    let fill_opacity = series_style(ctx.env()).fill_opacity();

    for line in lines {
        let options = &line.options;

        if options.fill == Fill::None {
            continue;
        }

        let (_, value_id) = line.orientation.index_and_value_scales(options);
        let Some(value_scale) = scales.get(value_id) else {
            continue;
        };

        let area = line.orientation.frame_area(chart_area);

        let Rgba(r, g, b, a) = Rgba::from(dataset_color(options, line.dataset_index, palette));
        let color = rgba(r, g, b, a * fill_opacity);

        for run in &line.runs {
            let points = &line.points[run.clone()];

            if points.len() < 2 {
                continue;
            }

            ctx.begin_path();
            let mut path = Oriented { path: &mut *ctx, orientation: line.orientation };
            trace_line(&mut path, points, options.interpolation, area);

            let below = match (options.fill, line.below) {
                (Fill::Stack, Some(below)) => Some(&lines[below]),
                _ => None,
            };

            if !below.is_some_and(|below| trace_lower_edge(&mut path, below, points, area)) {
                let baseline = baseline_pixel(options.fill, value_scale, line.orientation.value_axis(), chart_area);
                path.line_to(Position::new(points[points.len() - 1].x, baseline));
                path.line_to(Position::new(points[0].x, baseline));
            }

            ctx.close_path();
            ctx.set_fill_style(color);
            ctx.fill();
        }
    }
}

/// Traces the lower edge of a stacked area whose top is `top`, a run of the line above `below`:
/// back along `below`'s drawn points within `top`'s span along x, each of `below`'s runs with its
/// own interpolation, joined straight across its gaps. Returns `false`, tracing nothing, when
/// `below` has no points in that span. Everything is in the index frame, `area` included.
fn trace_lower_edge(path: &mut impl PathSink, below: &DrawnLine, top: &[Position], area: Rect) -> bool {
    let (left, right) = top.iter().fold((Scalar::INFINITY, Scalar::NEG_INFINITY), |(left, right), point| (left.min(point.x), right.max(point.x)));
    let mut traced = false;

    for run in below.runs.iter().rev() {
        // Half a pixel either way, so points on the span's ends count as inside despite rounding.
        let points: Vec<Position> = below.points[run.clone()]
            .iter()
            .filter(|point| point.x >= left - 0.5 && point.x <= right + 0.5)
            .copied()
            .collect();

        if !points.is_empty() {
            trace_line_reversed(path, &points, below.options.interpolation, area);
            traced = true;
        }
    }

    traced
}

/// Where a fill's baseline is along `value_axis`, the axis the values grow along: a y for upright
/// lines and an x for lying ones. Baseline values are included in the value scale's bounds, but a
/// scale can still end before one: a category scale with labels spans its labels, whatever the
/// data. A value outside the scale's range then fills to the nearest end of it, as bars grow from
/// the end nearest zero. `Fill::Bottom` is the start of the value axis and `Fill::Top` its end.
/// `Fill::Stack` without a line below fills to zero.
fn baseline_pixel(fill: Fill, value_scale: &dyn Scale, value_axis: Axis, chart_area: Rect) -> Scalar {
    match fill {
        Fill::Bottom => normalized_pixel(value_axis, 0.0, chart_area),
        Fill::Top => normalized_pixel(value_axis, 1.0, chart_area),
        fill => {
            let value = fill.baseline_value().unwrap_or(0.0);
            let (low, high) = (value_scale.min().min(value_scale.max()), value_scale.min().max(value_scale.max()));
            pixel(value_axis, value_scale, value.clamp(low, high), chart_area)
        }
    }
}

/// Draws each line through its points, in order, with its dataset's `interpolation` (straight,
/// smooth, monotone or stepped), each unbroken run on its own (see `SpanGaps`).
pub(crate) fn draw_lines(lines: &[DrawnLine], ctx: &mut CanvasContext, chart_area: Rect, palette: &Palette) {
    for line in lines {
        let area = line.orientation.frame_area(chart_area);

        ctx.begin_path();
        let mut path = Oriented { path: &mut *ctx, orientation: line.orientation };
        for run in &line.runs {
            trace_line(&mut path, &line.points[run.clone()], line.options.interpolation, area);
        }
        ctx.set_stroke_style(dataset_color(&line.options, line.dataset_index, palette));
        ctx.stroke();
    }
}

/// Draws a filled circle at each of each line's points that isn't missing, in its colour (see
/// `point_color`), for the lines whose dataset index `drawn` accepts.
pub(crate) fn draw_points(lines: &[DrawnLine], ctx: &mut CanvasContext, palette: &Palette, drawn: impl Fn(usize) -> bool) {
    for line in lines.iter().filter(|line| drawn(line.dataset_index)) {
        for ((point, color), point_index) in line.points.iter().zip(&line.colors).zip(line.point_indices.iter().copied()) {
            let point = line.orientation.swap(*point);
            ctx.begin_path();
            ctx.circle(point.x, point.y, 7.0);
            ctx.set_fill_style(point_color(*color, &line.options, line.dataset_index, point_index, palette));
            ctx.fill();
        }
    }
}

/// The colours of the chart's `SeriesStyle`, and how far into them a controller starts.
pub(crate) struct Palette {
    colors: Vec<Color>,
    offset: usize,
}

impl Palette {
    /// The palette of the `SeriesStyle` in `ctx`'s environment, for a controller starting at
    /// `offset`.
    pub(crate) fn new(ctx: &mut CanvasContext, offset: usize) -> Palette {
        Palette::from_env(ctx.env(), offset)
    }

    /// The palette of the `SeriesStyle` in `env`, for a controller starting at `offset`.
    pub(crate) fn from_env(env: &Environment, offset: usize) -> Palette {
        Palette { colors: series_style(env).palette(env), offset }
    }

    /// The colour for a dataset or point by its `index` in the controller, shifted by the
    /// controller's offset.
    fn by_index(&self, index: usize) -> Color {
        self.at(self.offset + index)
    }

    /// The colour at `index`, wrapping around at the end. An empty palette gives the default
    /// colour.
    fn at(&self, index: usize) -> Color {
        if self.colors.is_empty() {
            Color::default()
        } else {
            self.colors[index % self.colors.len()]
        }
    }
}

/// The dataset's own colour, or the `palette`'s colour for its index in the controller. This is
/// the colour of everything drawn once for the whole dataset, such as a line and its area, so a
/// dataset coloured `ByPoint` uses its palette colour here too.
pub(crate) fn dataset_color(options: &DataSetOptions, dataset_index: usize, palette: &Palette) -> Color {
    match options.color {
        DataColor::ByDataSet | DataColor::ByPoint => palette.by_index(dataset_index),
        DataColor::Palette(index) => palette.at(index),
        DataColor::Color(color) => color,
    }
}

/// The colour `color` stands for in a dataset: a fixed colour, or a palette colour, picked or by
/// the dataset's index, as for `dataset_color`. A colour by point, which needs a point, is the
/// dataset's colour here.
pub(crate) fn resolve_color(color: DataColor, options: &DataSetOptions, dataset_index: usize, palette: &Palette) -> Color {
    match color {
        DataColor::Color(color) => color,
        DataColor::Palette(index) => palette.at(index),
        DataColor::ByDataSet | DataColor::ByPoint => dataset_color(options, dataset_index, palette),
    }
}

/// The colour of one bar or point: the point's own colour if it sets one, otherwise its
/// dataset's. A dataset coloured `ByPoint` gives each point the palette colour for its index in
/// the dataset (`point_index`), as does a point coloured `ByPoint` itself.
pub(crate) fn point_color(color: DataColor, options: &DataSetOptions, dataset_index: usize, point_index: usize, palette: &Palette) -> Color {
    match (color, options.color) {
        (DataColor::Color(color), _) => color,
        (DataColor::Palette(index), _) => palette.at(index),
        (DataColor::ByPoint, _) | (DataColor::ByDataSet, DataColor::ByPoint) => palette.by_index(point_index),
        (DataColor::ByDataSet, _) => dataset_color(options, dataset_index, palette),
    }
}

/// Adds a legend entry for each of `datasets` that has a name: the entry the dataset's first mark
/// to give one asks for (`mark_entry`, which reads the marks with the context it is given), or
/// else the colour its line, area or bars are drawn in (see `dataset_color`), with the marker
/// `marker` gives for its index and options.
pub(crate) fn legend_items<T: AsDataSet + ?Sized, D: Sequence<T>>(
    datasets: &D,
    env: &Environment,
    palette: &Palette,
    items: &mut Vec<LegendItem>,
    marker: impl Fn(usize, &DataSetOptions) -> LegendMarker,
    mark_entry: impl Fn(&T, &LegendContext) -> Option<(Color, LegendMarker)>,
) {
    datasets.foreach(&mut |dataset_index, dataset| {
        let options = dataset.as_dataset().options();

        let Some(label) = options.label.clone() else {
            return;
        };

        let context = LegendContext { env, options: &options, dataset_index, palette };
        let (color, marker) = mark_entry(dataset, &context)
            .unwrap_or_else(|| (dataset_color(&options, dataset_index, palette), marker(dataset_index, &options)));

        items.push(LegendItem { label, color, marker });
    });
}

/// The legend entry of the first of a dataset's marks to give one, from `marks`, which calls its
/// argument with each mark's entry.
pub(crate) fn first_mark_entry(marks: impl FnOnce(&mut dyn FnMut(Option<(Color, LegendMarker)>))) -> Option<(Color, LegendMarker)> {
    let mut entry = None;
    marks(&mut |own| {
        if entry.is_none() {
            entry = own;
        }
    });
    entry
}

/// The x and y scales a dataset is bound to, or `None` when the chart has no scale with one of
/// those ids. The chart warns about unknown ids when it fits its scales (see `Chart::fit_scales`).
pub(crate) fn dataset_scales<'a>(scales: &'a Scales, options: &DataSetOptions) -> Option<(&'a dyn Scale, &'a dyn Scale)> {
    Some((scales.get(&options.x_scale)?, scales.get(&options.y_scale)?))
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Dimension, Position, Rect, Scalar};
    use carbide::environment::Environment;
    use crate::{Axis, CategoryScale, DatasetExt, Fill, LineController, LineDataSet, Scale, ScatterController, StackId};
    use crate::controller::DatasetController;
    use crate::scale::{ScaleBounds, ScaleId, Scales};
    use carbide::color::{BLUE, GREEN, RED, WHITE};
    use crate::{DataColor, DataSetOptions, Interpolation};
    use crate::controller::orientation::Orientation;
    use super::{baseline_pixel, dataset_color, drawn_lines, point_color, Palette};

    const SALES: StackId = StackId::new("sales");

    fn y_range(controller: &dyn DatasetController) -> Option<(f64, f64)> {
        let mut bounds = ScaleBounds::default();
        controller.collect_bounds(&mut bounds, &mut Environment::new());
        bounds.get(&ScaleId::Y).range()
    }

    /// A filled line includes its baseline value in the y bounds, so the area's base is on the
    /// chart; filling to the chart area's edges includes nothing. A scatter chart only does so
    /// when it draws lines, as only then is the area drawn.
    #[test]
    fn filled_lines_include_their_baseline() {
        let values = vec![100.0, 150.0, 200.0];

        assert_eq!(y_range(&LineController::new(values.clone())), Some((100.0, 200.0)));
        assert_eq!(y_range(&LineController::new(values.clone().fill(Fill::Origin))), Some((0.0, 200.0)));
        assert_eq!(y_range(&LineController::new(values.clone().fill(Fill::Value(250.0)))), Some((100.0, 250.0)));
        assert_eq!(y_range(&LineController::new(values.clone().fill(Fill::Bottom))), Some((100.0, 200.0)));

        assert_eq!(y_range(&ScatterController::new(values.clone().fill(Fill::Origin))), Some((100.0, 200.0)), "a scatter chart without lines draws no area");
        assert_eq!(y_range(&ScatterController::new(values.fill(Fill::Origin)).show_line(true)), Some((0.0, 200.0)));
    }

    /// Stacked lines report the running totals they are drawn at, with one sum for positive and
    /// negative values alike, so a negative value lowers the total; a stacked area fills down to
    /// zero, which is included.
    #[test]
    fn stacked_lines_report_their_totals() {
        let first = vec![10.0, 20.0].stack(SALES);
        let second = vec![5.0, -30.0].stack(SALES);

        // Totals: 10 and 15 at x = 0, 20 and -10 at x = 1.
        assert_eq!(y_range(&LineController::new((first.clone(), second.clone()))), Some((-10.0, 20.0)));
        assert_eq!(y_range(&ScatterController::new((first, second))), Some((-30.0, 20.0)), "scatter charts don't stack");

        let areas = (vec![10.0, 20.0].stack(SALES).fill(Fill::Stack), vec![5.0, 8.0].stack(SALES).fill(Fill::Stack));
        assert_eq!(y_range(&LineController::new(areas)), Some((0.0, 28.0)), "the bottom area fills down to zero");

        let band = (vec![10.0, 20.0].stack(SALES), vec![5.0, 8.0].stack(SALES).fill(Fill::Stack));
        assert_eq!(y_range(&LineController::new(band)), Some((10.0, 28.0)), "an area filled to the line below doesn't reach zero");
    }

    /// Each stacked line knows the line below it in its stack: not one in another stack or
    /// without a stack, and not an empty one, which has nothing to fill down to. Its points sit at
    /// the running totals.
    #[test]
    fn stacked_lines_know_the_line_below() {
        const OTHER: StackId = StackId::new("other");
        let empty: Vec<Scalar> = vec![];

        let controller = LineController::new((
            vec![10.0, 10.0].stack(SALES),
            vec![50.0, 50.0],
            empty.stack(SALES),
            vec![5.0, 5.0].stack(OTHER),
            vec![20.0, 20.0].stack(SALES),
        ));

        let mut bounds = ScaleBounds::default();
        controller.collect_bounds(&mut bounds, &mut Environment::new());
        let mut scales = Scales::new();
        scales.fit(&bounds);

        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(100.0, 100.0));
        let lines = drawn_lines::<dyn LineDataSet, _>(&controller.dataset_sequence, &scales, area, true, Orientation::Vertical, |_| true);

        let below: Vec<Option<usize>> = lines.iter().map(|line| line.below).collect();
        assert_eq!(below, vec![None, None, Some(0), None, Some(0)]);

        let y = scales.get(&ScaleId::Y).unwrap();
        let pixel = |value: Scalar| area.height() * (1.0 - y.normalize(value)) + area.bottom();
        assert_eq!(lines[4].points[0].y, pixel(30.0), "the top line should sit at 10 + 20, the total of its stack");
    }

    /// Datasets in a stack are matched by x value: where the dataset below has no point, the one
    /// above adds to nothing, so its line dips to its own value there.
    #[test]
    fn stacked_lines_dip_where_the_line_below_has_no_point() {
        let controller = LineController::new((
            vec![10.0, 10.0].stack(SALES),
            vec![5.0, 5.0, 5.0].stack(SALES),
        ));

        let mut bounds = ScaleBounds::default();
        controller.collect_bounds(&mut bounds, &mut Environment::new());
        let mut scales = Scales::new();
        scales.fit(&bounds);

        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(100.0, 100.0));
        let lines = drawn_lines::<dyn LineDataSet, _>(&controller.dataset_sequence, &scales, area, true, Orientation::Vertical, |_| true);

        let y = scales.get(&ScaleId::Y).unwrap();
        let pixel = |value: Scalar| area.height() * (1.0 - y.normalize(value)) + area.bottom();
        let top: Vec<Scalar> = lines[1].points.iter().map(|point| point.y).collect();
        assert_eq!(top, vec![pixel(15.0), pixel(15.0), pixel(5.0)], "the top line should dip to 5 where the line below ends");
    }

    /// A fill baseline outside the y scale's range fills to the nearest end of it. Baselines are
    /// included in the bounds, but a category scale with labels spans its labels whatever the
    /// data, so a value above the last label fills to the top of the scale.
    #[test]
    fn fill_baselines_outside_the_scale_fill_to_its_nearest_end() {
        let mut scale = CategoryScale::new(Axis::Vertical).labels(["Low", "Medium", "High", "Critical"]);
        scale.set_range(0.0, 10.0);
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(100.0, 100.0));

        let pixel = |value: Scalar| area.height() * (1.0 - scale.normalize(value)) + area.bottom();
        assert_eq!(baseline_pixel(Fill::Value(10.0), &scale, Axis::Vertical, area), pixel(3.0), "a value past the last label should fill to it");
        assert_eq!(baseline_pixel(Fill::Value(-5.0), &scale, Axis::Vertical, area), pixel(0.0), "a value before the first label should fill to it");
        assert_eq!(baseline_pixel(Fill::Value(2.0), &scale, Axis::Vertical, area), pixel(2.0));
    }

    fn options(color: DataColor) -> DataSetOptions {
        DataSetOptions {
            color,
            interpolation: Interpolation::Linear,
            fill: Fill::None,
            span_gaps: crate::SpanGaps::Never,
            x_scale: ScaleId::X,
            y_scale: ScaleId::Y,
            stack: None,
            label: None,
        }
    }

    /// A dataset coloured `ByPoint` gives each point the palette colour for its index in the
    /// dataset, wrapping at the palette's end, while what is drawn once for the whole dataset
    /// keeps the palette colour for the dataset's index. A point's own colour always wins.
    #[test]
    fn points_coloured_by_point_cycle_through_the_palette() {
        let by_point = options(DataColor::ByPoint);

        let palette = Palette { colors: vec![WHITE, GREEN, RED], offset: 0 };
        let colours: Vec<_> = (0..4).map(|point_index| point_color(DataColor::ByDataSet, &by_point, 1, point_index, &palette)).collect();
        assert_eq!(colours, vec![WHITE, GREEN, RED, WHITE], "by the point's index, not the dataset's");
        assert_eq!(dataset_color(&by_point, 1, &palette), GREEN, "the line and its area keep the dataset's palette colour");

        assert_eq!(point_color(DataColor::Color(BLUE), &by_point, 1, 2, &palette), BLUE, "a point's own colour wins");
        assert_eq!(point_color(DataColor::ByPoint, &options(DataColor::ByDataSet), 0, 2, &palette), RED, "a point can ask for its palette colour itself");
        assert_eq!(point_color(DataColor::ByDataSet, &options(DataColor::ByDataSet), 1, 2, &palette), GREEN, "otherwise points take the dataset's colour");
        assert_eq!(point_color(DataColor::ByDataSet, &options(DataColor::Color(BLUE)), 1, 2, &palette), BLUE);
    }

    /// A controller's palette offset shifts the colours its datasets and points take by their
    /// index; a dataset or point that picks a palette colour itself keeps it, and a fixed colour
    /// stays fixed. Palette indices wrap around at the end.
    #[test]
    fn palette_offsets_shift_colours_by_index_but_not_picked_ones() {
        let palette = Palette { colors: vec![WHITE, GREEN, RED], offset: 2 };

        assert_eq!(dataset_color(&options(DataColor::ByDataSet), 0, &palette), RED, "dataset 0 starts two colours in");
        assert_eq!(dataset_color(&options(DataColor::ByDataSet), 1, &palette), WHITE, "and wraps around");
        assert_eq!(point_color(DataColor::ByDataSet, &options(DataColor::ByPoint), 0, 1, &palette), WHITE, "points by index are shifted too");

        assert_eq!(dataset_color(&options(DataColor::Palette(1)), 0, &palette), GREEN, "a picked colour isn't shifted");
        assert_eq!(dataset_color(&options(DataColor::Palette(4)), 0, &palette), GREEN, "picked indices wrap around");
        assert_eq!(point_color(DataColor::Palette(0), &options(DataColor::ByDataSet), 1, 1, &palette), WHITE, "a point can pick a colour itself");
        assert_eq!(point_color(DataColor::ByDataSet, &options(DataColor::Palette(1)), 1, 1, &palette), GREEN, "its points take the dataset's picked colour");
        assert_eq!(dataset_color(&options(DataColor::Color(BLUE)), 0, &palette), BLUE);
    }

    /// A line breaks into runs at missing points with `SpanGaps::Never`, joins across them with
    /// `Always`, and with `Within` breaks only across gaps wider than the limit along x, also where
    /// no point is missing.
    #[test]
    fn lines_split_into_runs_at_gaps() {
        use super::split_runs;
        use crate::SpanGaps;

        // x: 0 1 _ 3 4 _ _ 7 8, then a point far away at 20 with nothing missing before it.
        let points = [Some(0.0), Some(1.0), None, Some(3.0), Some(4.0), None, None, Some(7.0), Some(8.0), Some(20.0)];

        assert_eq!(split_runs(&points, SpanGaps::Never), vec![0..2, 2..4, 4..7]);
        assert_eq!(split_runs(&points, SpanGaps::Always), vec![0..7]);
        assert_eq!(split_runs(&points, SpanGaps::Within(2.5)), vec![0..4, 4..6, 6..7], "gaps of 2 join; gaps of 3 and 12 break");

        assert_eq!(split_runs(&[None, Some(1.0), None], SpanGaps::Never), vec![0..1], "missing ends make no empty runs");
        assert_eq!(split_runs(&[None, None], SpanGaps::Never), Vec::<std::ops::Range<usize>>::new());
    }

    /// A missing value in a stacked line adds nothing to the stack and is left out of the bounds,
    /// instead of making them NaN.
    #[test]
    fn missing_values_add_nothing_to_a_stack() {
        let lower: Vec<Option<Scalar>> = vec![Some(10.0), None, Some(10.0)];
        let upper = vec![5.0, 5.0, 5.0].stack(SALES);

        assert_eq!(y_range(&LineController::new((lower.stack(SALES), upper))), Some((5.0, 15.0)), "the upper line dips to 5 where the lower one is missing");
    }

    /// Datasets of different point types share a controller, as points are read as numbers:
    /// plain values next to optional ones in one tuple.
    #[test]
    fn datasets_of_different_point_types_share_a_controller() {
        let plain = vec![1.0, 2.0, 3.0];
        let optional: Vec<Option<Scalar>> = vec![Some(10.0), None, Some(-4.0)];
        let points = vec![(0.0, 5.0), (2.0, 6.0)];

        assert_eq!(y_range(&LineController::new((plain, optional, points))), Some((-4.0, 10.0)));
    }
}
