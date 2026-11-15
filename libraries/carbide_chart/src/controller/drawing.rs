//! Drawing shared by the line and scatter controllers, which differ only in which parts they draw
//! by default: a line chart draws the line through each dataset's points and the points; a scatter
//! chart draws the points, and the line only with `show_line`. Where a line is drawn, so is the
//! area under it, for datasets that `fill`.

use carbide::color::{Color, GREEN, RED, WHITE};
use std::collections::{HashMap, HashSet};
use carbide::draw::{Position, Rect, Scalar};
use carbide::draw::color::{rgba, Rgba};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataSetOptions, DataSetSequence, Fill};
use crate::controller::line_path::{trace_line, trace_line_reversed};
use crate::controller::stacking::{StackKey, StackSums};
use crate::scale::{Scale, ScaleBounds, Scales};

/// The colours datasets get, by index, when they don't set one.
const PALETTE: [Color; 3] = [WHITE, GREEN, RED];

/// How opaque a filled area is, so the grid and the areas behind it show through.
const FILL_OPACITY: f32 = 0.3;

/// Reports every dataset's x and y range to the scales it is bound to. With `fills`, when the
/// controller draws lines, a filled dataset's baseline value is included too, so the area's base
/// is on the chart; a `Fill::Stack` area with a line below it fills to that line, so it includes
/// nothing. With `stacks`, a dataset in a stack reports the running totals it is drawn at instead
/// of its own values. Empty datasets have no bounds and contribute nothing.
pub(crate) fn collect_point_bounds<D>(datasets: &D, bounds: &mut ScaleBounds, fills: bool, stacks: bool)
where
    D: DataSetSequence<Scalar, Scalar, Scalar>,
{
    let mut sums = StackSums::single();
    // The stacks that already have a non-empty line, which a later `Fill::Stack` fills down to.
    let mut filled_stacks: HashSet<StackKey> = HashSet::new();

    datasets.foreach(&mut |dataset_index, dataset| {
        let (Some((x_min, y_min, _)), Some((x_max, y_max, _))) = (dataset.min(), dataset.max()) else {
            return;
        };

        let options = dataset.options();
        let key = StackKey::new(&options, dataset_index);
        let has_line_below = stacks && matches!(key, StackKey::Stacked { .. }) && !filled_stacks.insert(key.clone());

        bounds.include(&options.x_scale, x_min, x_max);

        if stacks {
            dataset.points(&mut |_index, point| {
                let (_, total) = sums.add(&key, point.x(), point.y());
                bounds.include(&options.y_scale, total, total);
            });
        } else {
            bounds.include(&options.y_scale, y_min, y_max);
        }

        let fills_to_line = options.fill == Fill::Stack && has_line_below;

        if fills && !fills_to_line && let Some(baseline) = options.fill.baseline_value() {
            bounds.include(&options.y_scale, baseline, baseline);
        }
    });
}

/// One dataset's line as it is drawn: its points in pixels, at their running totals when it is
/// stacked, and the line below it in its stack, which a `Fill::Stack` area fills down to.
pub(crate) struct DrawnLine {
    dataset_index: usize,
    options: DataSetOptions,
    points: Vec<Position>,
    colors: Vec<DataColor>,
    /// The index, in the controller's lines, of the line below this one in its stack.
    below: Option<usize>,
}

/// The lines of a controller's datasets, in dataset order, mapped through their scales into
/// `chart_area`. With `stacks`, datasets in the same stack are drawn at their running totals,
/// one sum per stack (see `StackSums::single`), and each knows the line below it. Datasets bound
/// to a scale the chart doesn't have are left out.
pub(crate) fn drawn_lines<D>(datasets: &D, scales: &Scales, chart_area: Rect, stacks: bool) -> Vec<DrawnLine>
where
    D: DataSetSequence<Scalar, Scalar, Scalar>,
{
    let mut lines: Vec<DrawnLine> = vec![];
    let mut sums = StackSums::single();
    let mut tops: HashMap<StackKey, usize> = HashMap::new();

    datasets.foreach(&mut |dataset_index, dataset| {
        let options = dataset.options();

        let Some((x_scale, y_scale)) = dataset_scales(scales, &options) else {
            return;
        };

        let key = StackKey::new(&options, dataset_index);
        let mut points = vec![];
        let mut colors = vec![];

        dataset.points(&mut |_index, point| {
            let y = if stacks { sums.add(&key, point.x(), point.y()).1 } else { point.y() };

            points.push(Position::new(
                chart_area.width() * x_scale.normalize(point.x()) + chart_area.left(),
                chart_area.height() * (1.0 - y_scale.normalize(y)) + chart_area.bottom(),
            ));
            colors.push(point.color());
        });

        let below = if stacks && matches!(key, StackKey::Stacked { .. }) { tops.get(&key).copied() } else { None };

        // An empty line can't be filled down to, so the next one fills to the line below it.
        if stacks && !points.is_empty() {
            tops.insert(key, lines.len());
        }

        lines.push(DrawnLine { dataset_index, options, points, colors, below });
    });

    lines
}

/// Fills the area between each filled line and its `Fill` baseline, in the dataset's colour,
/// translucent. The area's top follows the line exactly, through `trace_line`, then closes along
/// the baseline, or, for `Fill::Stack`, back along the line below it in its stack, traced in
/// reverse with that line's own interpolation, so the area fits exactly between the two drawn
/// lines. A line needs two points to have an area.
pub(crate) fn draw_fills(lines: &[DrawnLine], ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect) {
    for line in lines {
        let options = &line.options;

        if options.fill == Fill::None || line.points.len() < 2 {
            continue;
        }

        let Some(y_scale) = scales.get(&options.y_scale) else {
            continue;
        };

        let (first, last) = (line.points[0], line.points[line.points.len() - 1]);

        ctx.begin_path();
        trace_line(ctx, &line.points, options.interpolation, chart_area);

        match (options.fill, line.below) {
            (Fill::Stack, Some(below)) => {
                let below = &lines[below];
                trace_line_reversed(ctx, &below.points, below.options.interpolation, chart_area);
            }
            (fill, _) => {
                let baseline = baseline_pixel(fill, y_scale, chart_area);
                ctx.line_to(last.x, baseline);
                ctx.line_to(first.x, baseline);
            }
        }

        ctx.close_path();

        let Rgba(r, g, b, a) = Rgba::from(dataset_color(options, line.dataset_index));
        ctx.set_fill_style(rgba(r, g, b, a * FILL_OPACITY));
        ctx.fill();
    }
}

/// The y pixel of a fill's baseline. Baseline values are included in the y bounds, but a scale
/// can still end before one: a category scale with labels spans its labels, whatever the data.
/// A value outside the scale's range then fills to the nearest end of it, as bars grow from the
/// end nearest zero. `Fill::Stack` without a line below fills to zero.
fn baseline_pixel(fill: Fill, y_scale: &dyn Scale, chart_area: Rect) -> Scalar {
    // `Rect`'s top and bottom are swapped on screen (see `Scale::draw_grid`): the larger y is the
    // bottom edge.
    match fill {
        Fill::Bottom => chart_area.top().max(chart_area.bottom()),
        Fill::Top => chart_area.top().min(chart_area.bottom()),
        fill => {
            let value = fill.baseline_value().unwrap_or(0.0);
            let (low, high) = (y_scale.min().min(y_scale.max()), y_scale.min().max(y_scale.max()));
            chart_area.height() * (1.0 - y_scale.normalize(value.clamp(low, high))) + chart_area.bottom()
        }
    }
}

/// Draws each line through its points, in order, with its dataset's `interpolation` (straight,
/// smooth, monotone or stepped).
pub(crate) fn draw_lines(lines: &[DrawnLine], ctx: &mut CanvasContext, chart_area: Rect) {
    for line in lines {
        ctx.begin_path();
        trace_line(ctx, &line.points, line.options.interpolation, chart_area);
        ctx.set_stroke_style(dataset_color(&line.options, line.dataset_index));
        ctx.stroke();
    }
}

/// Draws a filled circle at each of each line's points, in its colour (see `point_color`).
pub(crate) fn draw_points(lines: &[DrawnLine], ctx: &mut CanvasContext) {
    for line in lines {
        for (point_index, (point, color)) in line.points.iter().zip(&line.colors).enumerate() {
            ctx.begin_path();
            ctx.circle(point.x, point.y, 7.0);
            ctx.set_fill_style(point_color(*color, &line.options, line.dataset_index, point_index));
            ctx.fill();
        }
    }
}

/// The dataset's own colour, or the palette's colour for its index in the controller. This is
/// the colour of everything drawn once for the whole dataset, such as a line and its area, so a
/// dataset coloured `ByPoint` uses its palette colour here too.
pub(crate) fn dataset_color(options: &DataSetOptions, dataset_index: usize) -> Color {
    match options.color {
        DataColor::ByDataSet | DataColor::ByPoint => palette_color(dataset_index),
        DataColor::Color(color) => color,
    }
}

/// The colour of one bar or point: the point's own colour if it sets one, otherwise its
/// dataset's. A dataset coloured `ByPoint` gives each point the palette colour for its index in
/// the dataset (`point_index`), as does a point coloured `ByPoint` itself.
pub(crate) fn point_color(color: DataColor, options: &DataSetOptions, dataset_index: usize, point_index: usize) -> Color {
    match (color, options.color) {
        (DataColor::Color(color), _) => color,
        (DataColor::ByPoint, _) | (DataColor::ByDataSet, DataColor::ByPoint) => palette_color(point_index),
        (DataColor::ByDataSet, _) => dataset_color(options, dataset_index),
    }
}

/// The palette's colour for `index`, wrapping around at its end.
fn palette_color(index: usize) -> Color {
    PALETTE[index % PALETTE.len()]
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
    use crate::{Axis, CategoryScale, DatasetExt, Fill, LineController, Scale, ScatterController, StackId};
    use crate::controller::DatasetController;
    use crate::scale::{ScaleBounds, ScaleId, Scales};
    use carbide::color::{BLUE, GREEN, RED, WHITE};
    use crate::{DataColor, DataSetOptions, Interpolation};
    use super::{baseline_pixel, dataset_color, drawn_lines, point_color};

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
        let lines = drawn_lines(&controller.dataset_sequence, &scales, area, true);

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
        let lines = drawn_lines(&controller.dataset_sequence, &scales, area, true);

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
        assert_eq!(baseline_pixel(Fill::Value(10.0), &scale, area), pixel(3.0), "a value past the last label should fill to it");
        assert_eq!(baseline_pixel(Fill::Value(-5.0), &scale, area), pixel(0.0), "a value before the first label should fill to it");
        assert_eq!(baseline_pixel(Fill::Value(2.0), &scale, area), pixel(2.0));
    }

    fn options(color: DataColor) -> DataSetOptions {
        DataSetOptions {
            color,
            interpolation: Interpolation::Linear,
            fill: Fill::None,
            x_scale: ScaleId::X,
            y_scale: ScaleId::Y,
            stack: None,
        }
    }

    /// A dataset coloured `ByPoint` gives each point the palette colour for its index in the
    /// dataset, wrapping at the palette's end, while what is drawn once for the whole dataset
    /// keeps the palette colour for the dataset's index. A point's own colour always wins.
    #[test]
    fn points_coloured_by_point_cycle_through_the_palette() {
        let by_point = options(DataColor::ByPoint);

        let colours: Vec<_> = (0..4).map(|point_index| point_color(DataColor::ByDataSet, &by_point, 1, point_index)).collect();
        assert_eq!(colours, vec![WHITE, GREEN, RED, WHITE], "by the point's index, not the dataset's");
        assert_eq!(dataset_color(&by_point, 1), GREEN, "the line and its area keep the dataset's palette colour");

        assert_eq!(point_color(DataColor::Color(BLUE), &by_point, 1, 2), BLUE, "a point's own colour wins");
        assert_eq!(point_color(DataColor::ByPoint, &options(DataColor::ByDataSet), 0, 2), RED, "a point can ask for its palette colour itself");
        assert_eq!(point_color(DataColor::ByDataSet, &options(DataColor::ByDataSet), 1, 2), GREEN, "otherwise points take the dataset's colour");
        assert_eq!(point_color(DataColor::ByDataSet, &options(DataColor::Color(BLUE)), 1, 2), BLUE);
    }
}
