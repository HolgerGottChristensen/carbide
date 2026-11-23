//! Drawing shared by the line and scatter controllers, which differ only in which parts they draw
//! by default: a line chart draws the line through each dataset's points and the points; a scatter
//! chart draws the points, and the line only with `show_line`. Where a line is drawn, so is the
//! area under it, for datasets that `fill`.

use carbide::color::Color;
use std::collections::{HashMap, HashSet};
use std::ops::{Range, RangeInclusive};
use std::rc::Rc;
use carbide::draw::{Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::{DataColor, DataSetOptions, Fill, PointValue, SpanGaps, StackId, Value};
use crate::mark::{AsDataSet, LegendContext};
use crate::sequence::Sequence;
use crate::controller::line_path::{trace_line, trace_line_reversed, PathSink};
use crate::controller::polar_path::{trace_polar_line, trace_polar_line_reversed, PolarPoint};
use crate::controller::orientation::{is_polar, normalized_pixel, pixel, Oriented, Orientation};
use crate::controller::stacking::{normalized_totals, StackKey, StackSums};
use crate::scale::{Axis, Coordinates, Polar, PolarRole, Scale, ScaleBounds, ScaleId, Scales, System};
use crate::scale::axis::{angle_corners, format_number, tick_label};
use crate::scale::polar::{Ring, RingShape};
use crate::scale::GridShape;
use crate::legend::{LegendItem, LegendMarker};
use crate::colors::{DatasetColors, SeriesColors};
use crate::geometry::{Along, Datum, DatumKey, Polyline, Shape, Shapes};
use crate::interaction::{DatumInfo, ValueText};
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
    let (placement, requests) = bounds.split();
    let totals = normalized_totals(datasets, placement, orientation, &counted);
    let mut sums = StackSums::single().normalized(totals);
    // The stacks that already have a non-empty line, which a later `Fill::Stack` fills down to.
    let mut filled_stacks: HashSet<StackKey> = HashSet::new();

    datasets.foreach(&mut |dataset_index, dataset| {
        if !counted(dataset_index) {
            return;
        }

        let dataset = dataset.as_dataset();
        let options = dataset.options();
        // A hidden dataset adds nothing to the bounds, its stack or what is drawn.
        if !options.visible {
            return;
        }
        let (index_scale, value_scale) = orientation.for_dataset(&options, placement).index_and_value_scales(&options);
        let coordinates = placement.coordinates(index_scale, value_scale);
        let key = StackKey::new(&options, dataset_index);
        let mut has_points = false;

        dataset.points(&mut |_index, point| {
            let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) else {
                return;
            };

            has_points = true;
            let y = if stacks { sums.add(&key, x, y).1 } else { y };
            requests.include_point(index_scale, value_scale, x, y);
        });

        // An empty dataset adds nothing, not even its fill's baseline, and isn't a line to fill to.
        if !has_points {
            return;
        }

        // A normalised stack's shares run from zero, as bars from zero, so the value scale shows the
        // whole 0 to 100.
        if stacks && options.stack.as_ref().is_some_and(StackId::is_normalized) {
            requests.include(value_scale, 0.0, 0.0);
        }

        let has_line_below = stacks && matches!(key, StackKey::Stacked { .. }) && !filled_stacks.insert(key.clone());

        let fills_to_line = options.fill == Fill::Stack && has_line_below;

        if fills && !fills_to_line && let Some(baseline) = options.fill.baseline_value() {
            requests.include(value_scale, baseline, baseline);
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
    /// For a dataset on polar scales, how its points lie around the circle. Its `points` are
    /// then on the screen, and its orientation upright, so they are drawn where they are.
    polar: Option<PolarLine>,
    /// The index scale the points' x values are on.
    index_id: ScaleId,
    /// Each drawn point's x value on the index scale.
    xs: Vec<Scalar>,
}

impl DrawnLine {
    /// The colours of the line's dataset, among the controller's `colors`.
    fn dataset_colors<'a>(&self, colors: &'a SeriesColors) -> DatasetColors<'a> {
        colors.dataset(self.options.color, self.dataset_index)
    }

    /// The datum the drawn point at `point` shows, and where it sits along the index axis.
    fn datum(&self, point: usize, shapes: &mut Shapes) -> Datum {
        let along = match &self.polar {
            Some(polar) => {
                let center = polar.polar.circle().center;
                let point = polar.points[point];
                if polar.values_on_radius {
                    Along::Angle { center, angle: point.angle }
                } else {
                    Along::Distance { center, distance: point.distance }
                }
            }
            // In the index frame, x is along the index axis.
            None => match self.orientation {
                Orientation::Vertical => Along::X(self.points[point].x),
                Orientation::Horizontal => Along::Y(self.points[point].x),
            },
        };

        Datum { index: self.point_indices[point], place: Some(shapes.place(&self.index_id, self.xs[point], along)) }
    }

    /// The drawn point at `point` on the screen.
    fn screen(&self, point: usize) -> Position {
        self.orientation.swap(self.points[point])
    }
}

/// How a polar line's points lie around the circle (see `controller/polar_path.rs`).
pub(crate) struct PolarLine {
    polar: Polar,
    /// The line's points that aren't missing, as `DrawnLine::points`, with their screen angles
    /// and distances from the centre.
    points: Vec<PolarPoint>,
    /// Whether the line closes into a loop: on a full circle, with its categories around it, and
    /// not broken where it would close (see `closes`).
    closed: bool,
    /// Whether the values run out along the radius, as in a radar. When the categories run out
    /// along it instead (`horizontal`), the line isn't filled.
    values_on_radius: bool,
    /// The spokes of the angle scale, when the radius scale's rings are polygons, so an area
    /// filled to a ring fills to the polygon the grid draws.
    corners: Option<Rc<[Scalar]>>,
}

impl PolarLine {
    /// The first point again, a full turn on, for a loop to close on.
    fn closing(&self, run: &Range<usize>) -> Option<PolarPoint> {
        if !self.closed {
            return None;
        }

        let first = self.points[run.start];
        let (start, end) = self.polar.angle_range();
        let turn = if end >= start { std::f64::consts::TAU } else { -std::f64::consts::TAU };
        Some(PolarPoint { angle: first.angle + turn, ..first })
    }
}

/// Whether a polar line with these `runs`, from points whose x values or gaps are `xs`, closes
/// into a loop: when it is drawn in one run, unless a missing value at either end breaks it there,
/// which only `SpanGaps::Never` does (see `split_runs`).
pub(crate) fn closes(runs: &[Range<usize>], xs: &[Option<Scalar>], span_gaps: SpanGaps) -> bool {
    let broken_at_ends = span_gaps == SpanGaps::Never && (xs.first() == Some(&None) || xs.last() == Some(&None));
    runs.len() == 1 && runs[0].len() >= 2 && !broken_at_ends
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

/// Which of a dataset's points to lay out on a zoomed index scale, from `points`, each point
/// that isn't missing with its normalized position along the scale, in order: the points inside
/// the window, and one either side, so lines run off the chart area's edges, as a range of point
/// indices; when none is inside, the two either side of the window, if a line between them crosses
/// it, else none. `None`, for all of them, when the points aren't in order along the scale.
pub(crate) fn visible_points(points: &[(usize, Scalar)]) -> Option<RangeInclusive<usize>> {
    let positions = || points.iter().map(|(_, position)| *position);
    let rising = positions().zip(positions().skip(1)).all(|(a, b)| a <= b);
    let falling = positions().zip(positions().skip(1)).all(|(a, b)| a >= b);
    if !rising && !falling {
        return None;
    }

    let inside = |position: Scalar| (-1e-9..=1.0 + 1e-9).contains(&position);
    let first = points.iter().position(|(_, position)| inside(*position));
    let last = points.iter().rposition(|(_, position)| inside(*position));

    let none = 1..=0;
    match (first, last) {
        (Some(first), Some(last)) => Some(points[first.saturating_sub(1)].0..=points[(last + 1).min(points.len() - 1)].0),
        _ => Some(
            points.windows(2)
                .find(|pair| (pair[0].1 < 0.0 && pair[1].1 > 1.0) || (pair[0].1 > 1.0 && pair[1].1 < 0.0))
                .map_or(none, |pair| pair[0].0..=pair[1].0)
        ),
    }
}

/// The points of `dataset` to lay out with `index_scale`, where it places them through
/// `coordinates`: those `visible_points` keeps when the scale is zoomed, or `None`, for all.
pub(crate) fn zoomed_points(dataset: &dyn crate::DataSet, coordinates: &Coordinates, index_scale: &dyn Scale) -> Option<RangeInclusive<usize>> {
    if index_scale.axis_options().window().is_whole() {
        return None;
    }

    let mut points = vec![];
    dataset.points(&mut |point_index, point| {
        if let (Some(x), Some(_)) = (coordinates.x(point), coordinates.y(point)) {
            points.push((point_index, index_scale.normalize(x)));
        }
    });

    visible_points(&points)
}

/// The lines of a controller's datasets, in dataset order, mapped through their scales into
/// `chart_area`, each point's x along its index scale and its y along its value scale, as
/// `orientation` picks them. With `stacks`, datasets in the same stack are drawn at their running
/// totals, one sum per stack (see `StackSums::single`), and each knows the line below it.
/// Datasets bound to a scale the chart doesn't have are left out, and so are the datasets
/// `counted` turns down, whose lines a mark replaces; they don't stack either. With `loops`, a
/// line around a full circle closes into a loop (see `closes`), as a radar's does; scatter lines
/// stay open, as scattered data isn't a loop.
pub(crate) fn drawn_lines<T, D>(datasets: &D, scales: &Scales, chart_area: Rect, stacks: bool, loops: bool, orientation: Orientation, counted: impl Fn(usize) -> bool) -> Vec<DrawnLine>
where
    T: AsDataSet + ?Sized,
    D: Sequence<T>,
{
    let mut lines: Vec<DrawnLine> = vec![];
    let mut sums = StackSums::single().normalized(normalized_totals(datasets, scales, orientation, &counted));
    let mut tops: HashMap<StackKey, usize> = HashMap::new();

    datasets.foreach(&mut |dataset_index, dataset| {
        if !counted(dataset_index) {
            return;
        }

        let dataset = dataset.as_dataset();
        let options = dataset.options();
        // A hidden dataset adds nothing to the bounds, its stack or what is drawn.
        if !options.visible {
            return;
        }

        // A polar dataset keeps its x on its x scale; its controller's orientation picks the roles.
        let wants_x_angle = orientation.polar_roles().0 == PolarRole::Angle;
        let orientation = orientation.for_dataset(&options, scales);
        let (index_id, value_id) = orientation.index_and_value_scales(&options);
        let (Some(index_scale), Some(value_scale)) = (scales.get(index_id), scales.get(value_id)) else {
            return;
        };

        // Around the circle, the angle is whichever of the two scales has that role.
        let (polar, corners) = match polar_frame(scales, index_id, value_id, wants_x_angle) {
            None => return,
            Some(None) => (None, None),
            Some(Some(frame)) => (Some((frame.polar, frame.x_is_angle)), frame.corners),
        };

        let key = StackKey::new(&options, dataset_index);
        let coordinates = Coordinates::new(scales, index_id, value_id);
        let mut polar_points = vec![];
        let mut points = vec![];
        let mut colors = vec![];
        let mut point_indices = vec![];
        let mut xs = vec![];
        let mut index_values = vec![];

        // On a zoomed scale along a side, only the points in its window and either side of it.
        let kept = if polar.is_none() { zoomed_points(dataset, &coordinates, index_scale) } else { None };

        dataset.points(&mut |point_index, point| {
            if kept.as_ref().is_some_and(|kept| !kept.contains(&point_index)) {
                return;
            }

            // A missing point isn't drawn, adds nothing to its stack, and may break the line.
            let (Some(x), Some(y)) = (coordinates.x(point), coordinates.y(point)) else {
                xs.push(None);
                return;
            };

            let y = if stacks { sums.add(&key, x, y).1 } else { y };

            match polar {
                Some((polar, index_on_angle)) => {
                    let (index, value) = (index_scale.normalize(x), value_scale.normalize(y));
                    let (angle, radius) = if index_on_angle { (index, value) } else { (value, index) };
                    let point = PolarPoint {
                        position: polar.position(angle, radius),
                        angle: polar.angle(angle),
                        distance: polar.distance(radius),
                    };
                    points.push(point.position);
                    polar_points.push(point);
                }
                None => points.push(Position::new(
                    pixel(orientation.index_axis(), index_scale, x, chart_area),
                    pixel(orientation.value_axis(), value_scale, y, chart_area),
                )),
            }
            colors.push(point.color());
            point_indices.push(point_index);
            index_values.push(x);
            xs.push(Some(x));
        });

        let runs = split_runs(&xs, options.span_gaps);

        let below = if stacks && matches!(key, StackKey::Stacked { .. }) { tops.get(&key).copied() } else { None };

        // An empty line can't be filled down to, so the next one fills to the line below it.
        if stacks && !points.is_empty() {
            tops.insert(key, lines.len());
        }

        let polar = polar.map(|(polar, index_on_angle)| PolarLine {
            polar,
            points: polar_points,
            closed: loops && index_on_angle && polar.is_full_circle() && closes(&runs, &xs, options.span_gaps),
            values_on_radius: index_on_angle,
            corners,
        });

        // Polar points are on the screen already.
        let orientation = if polar.is_some() { Orientation::Vertical } else { orientation };

        let index_id = index_id.clone();
        lines.push(DrawnLine { dataset_index, options, orientation, points, colors, point_indices, runs, below, polar, index_id, xs: index_values });
    });

    lines
}

/// How a polar dataset lies around the chart's circle: the mapping onto it, whether its x scale
/// is the angle, and the corners of the radius scale's rings when they are polygons, which steps
/// and fills follow (see `RingShape`).
pub(crate) struct PolarFrame {
    pub(crate) polar: Polar,
    pub(crate) x_is_angle: bool,
    pub(crate) corners: Option<Rc<[Scalar]>>,
}

impl PolarFrame {
    /// The shape of the rings around the circle.
    pub(crate) fn rings(&self) -> RingShape<'_> {
        RingShape { polar: &self.polar, corners: self.corners.as_deref() }
    }
}

/// How a dataset on `x_id` and `y_id` is drawn: `Some(None)` along the sides of the chart area,
/// `Some(Some(frame))` around its circle, or `None` when it isn't drawn: its scales belong to no
/// one system, or took the other roles than its controller asked for (`wants_x_angle`), which is
/// logged once, or no circle was fitted yet.
pub(crate) fn polar_frame(scales: &Scales, x_id: &ScaleId, y_id: &ScaleId, wants_x_angle: bool) -> Option<Option<PolarFrame>> {
    let x_is_angle = match scales.system(x_id, y_id)? {
        System::Cartesian => return Some(None),
        System::Polar { x_is_angle } if x_is_angle != wants_x_angle => {
            warn_other_roles(scales, x_id, y_id);
            return None;
        }
        System::Polar { x_is_angle } => x_is_angle,
    };

    let (angle_id, radius_id) = if x_is_angle { (x_id, y_id) } else { (y_id, x_id) };
    let polar = scales.polar(angle_id, radius_id)?;
    let corners = angle_corners(scales.get(angle_id)?);
    let corners = (scales.grid_shape(radius_id) == GridShape::Polygon && corners.len() >= 3).then(|| Rc::from(corners));

    Some(Some(PolarFrame { polar, x_is_angle, corners }))
}

/// Logs, once per pair, that a polar dataset on `x_id` and `y_id` isn't drawn because its scales
/// took the other roles than its controller asked for: another dataset asked first (see
/// `ScaleRequests::request_polar_role`).
pub(crate) fn warn_other_roles(scales: &Scales, x_id: &ScaleId, y_id: &ScaleId) {
    scales.warn_once("other roles", x_id, y_id, || format!("A dataset on the radial scales {:?} and {:?} is not drawn: they took the other roles, angle and radius, than its controller asked for, as another dataset asked first", x_id.name(), y_id.name()));
}

/// Fills the area between each run of each filled line and its `Fill` baseline, in the dataset's
/// colour, at the `SeriesStyle`'s fill opacity. The area's top follows the run exactly, through
/// `trace_line`, then closes along the baseline, or, for `Fill::Stack`, back along the line below
/// it in its stack, traced in reverse with that line's own interpolation, so the area fits
/// exactly between the two drawn lines. A run needs two points to have an area.
pub(crate) fn draw_fills(lines: &[DrawnLine], ctx: &mut CanvasContext, scales: &Scales, chart_area: Rect, colors: &SeriesColors) {
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

        let color = colors.translucent(line.dataset_colors(colors).color());

        if let Some(polar_line) = &line.polar {
            draw_polar_fill(line, polar_line, lines, value_scale, ctx, color);
            continue;
        }

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

/// Fills a polar line's area: between the line and a ring around the centre, in the shape of the
/// grid's rings (the inner radius for `Fill::Bottom`, the rim for `Fill::Top`, or the ring at the
/// baseline value), or, for
/// `Fill::Stack`, the line below it in its stack. A loop and a whole ring, or two loops, make a
/// ring-shaped area, traced in opposite directions so it fills under either fill rule; an open
/// line closes along the ring's arc, or back along the line below. Lines whose categories run out
/// along the radius aren't filled.
fn draw_polar_fill(line: &DrawnLine, polar_line: &PolarLine, lines: &[DrawnLine], value_scale: &dyn Scale, ctx: &mut CanvasContext, color: Color) {
    if !polar_line.values_on_radius {
        return;
    }

    let polar = &polar_line.polar;
    let center = polar.circle().center;
    let interpolation = line.options.interpolation;
    let rings = RingShape { polar, corners: polar_line.corners.as_deref() };

    let below = match (line.options.fill, line.below) {
        (Fill::Stack, Some(below)) => lines[below].polar.as_ref().map(|below_polar| (&lines[below], below_polar)),
        _ => None,
    };

    // The distance of the ring the area fills to, when not to the line below.
    let baseline = match line.options.fill {
        Fill::Bottom => polar.inner_radius(),
        Fill::Top => polar.circle().radius,
        fill => {
            let value = fill.baseline_value().unwrap_or(0.0);
            let (low, high) = (value_scale.min().min(value_scale.max()), value_scale.min().max(value_scale.max()));
            polar.distance(value_scale.normalize(value.clamp(low, high)))
        }
    };

    let ring = Ring { polar, distance: baseline, corners: polar_line.corners.as_deref() };

    for run in &line.runs {
        let points = &polar_line.points[run.clone()];
        if points.len() < 2 {
            continue;
        }

        ctx.begin_path();
        let closing = polar_line.closing(run);
        trace_polar_line(&mut *ctx, points, closing, interpolation, rings);

        match (closing, below) {
            (Some(_), Some((below, below_polar))) if below_polar.closed => {
                // The loop below, backwards, as a second loop.
                let reversed: Vec<PolarPoint> = below_polar.points.iter().rev().copied().collect();
                let closing = below_polar.closing(&(0..below_polar.points.len())).map(|first| {
                    let last = reversed[0];
                    PolarPoint { angle: last.angle - (first.angle - below_polar.points[0].angle), ..last }
                });
                let interpolation = reversed_steps(below.options.interpolation);
                let below_rings = RingShape { polar: &below_polar.polar, corners: below_polar.corners.as_deref() };
                trace_polar_line(&mut *ctx, &reversed, closing, interpolation, below_rings);
            }
            (None, Some((below, below_polar))) => {
                let below_rings = RingShape { polar: &below_polar.polar, corners: below_polar.corners.as_deref() };
                trace_polar_line_reversed(&mut *ctx, &below_polar.points, below.options.interpolation, below_rings);
            }
            (Some(_), _) => {
                // A whole ring, the other way round.
                if baseline > 0.5 {
                    ring.trace_whole(&mut *ctx, true);
                }
            }
            (None, None) => {
                let (first, last) = (points[0], points[points.len() - 1]);
                if baseline > 0.5 {
                    PathSink::line_to(&mut *ctx, ring.point(last.angle));
                    ring.trace(&mut *ctx, last.angle, first.angle);
                } else {
                    PathSink::line_to(&mut *ctx, center);
                }
            }
        }

        ctx.close_path();
        ctx.set_fill_style(color);
        ctx.fill();
    }
}

/// The interpolation a line traced backwards uses to follow the same path: steps turn the other
/// way round.
fn reversed_steps(interpolation: crate::Interpolation) -> crate::Interpolation {
    use crate::{Interpolation, Stepped};
    match interpolation {
        Interpolation::Stepped(Stepped::Before) => Interpolation::Stepped(Stepped::After),
        Interpolation::Stepped(Stepped::After) => Interpolation::Stepped(Stepped::Before),
        interpolation => interpolation,
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
/// lines and an x for horizontal ones. Baseline values are included in the value scale's bounds, but a
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
pub(crate) fn draw_lines(lines: &[DrawnLine], ctx: &mut CanvasContext, chart_area: Rect, colors: &SeriesColors) {
    for line in lines {
        if let Some(polar_line) = &line.polar {
            ctx.begin_path();
            for run in &line.runs {
                let closing = polar_line.closing(run);
                let rings = RingShape { polar: &polar_line.polar, corners: polar_line.corners.as_deref() };
                trace_polar_line(&mut *ctx, &polar_line.points[run.clone()], closing, line.options.interpolation, rings);
            }
            ctx.set_stroke_style(line.dataset_colors(colors).color());
            ctx.stroke();
            continue;
        }

        let area = line.orientation.frame_area(chart_area);

        ctx.begin_path();
        let mut path = Oriented { path: &mut *ctx, orientation: line.orientation };
        for run in &line.runs {
            trace_line(&mut path, &line.points[run.clone()], line.options.interpolation, area);
        }
        ctx.set_stroke_style(line.dataset_colors(colors).color());
        ctx.stroke();
    }
}

/// Draws a filled circle at each of each line's points that isn't missing, in its colour (see
/// `DatasetColors::point`), for the lines whose dataset index `drawn` accepts, each a marker in
/// `shapes`.
pub(crate) fn draw_points(lines: &[DrawnLine], ctx: &mut CanvasContext, colors: &SeriesColors, shapes: &mut Shapes, drawn: impl Fn(usize) -> bool) {
    for line in lines.iter().filter(|line| drawn(line.dataset_index)) {
        let dataset_colors = line.dataset_colors(colors);
        for point in 0..line.points.len() {
            let color = dataset_colors.point(line.point_indices[point], line.colors[point]);
            let datum = line.datum(point, shapes);
            let marker = Shape::Marker { center: line.screen(point), radius: POINT_RADIUS };
            let item = shapes.add(line.dataset_index, Some(datum), color, marker);

            ctx.begin_path();
            item.shape.trace(ctx);
            ctx.set_fill_style(item.color);
            ctx.fill();
        }
    }
}

/// The radius of a line's points.
const POINT_RADIUS: Scalar = 3.5;

/// Adds each line to `shapes` as a polyline through its points, with the areas it fills, for
/// picking: the lines a controller draws, in the order it draws them.
pub(crate) fn add_lines(lines: &[DrawnLine], shapes: &mut Shapes, scales: &Scales, chart_area: Rect, colors: &SeriesColors) {
    for line in lines {
        let vertices = (0..line.points.len()).map(|point| (line.screen(point), line.datum(point, shapes))).collect();
        let areas = if line.options.fill == Fill::None { vec![] } else { fill_areas(line, lines, scales, chart_area) };
        let polyline = Polyline {
            vertices,
            runs: line.runs.clone(),
            closed: line.polar.as_ref().is_some_and(|polar| polar.closed),
            areas,
        };

        shapes.add(line.dataset_index, None, line.dataset_colors(colors).color(), Shape::Polyline(polyline));
    }
}

/// The areas a filled line fills, as polygons on the screen through its points, straight between
/// them, for picking (see `Polyline::areas`): down to its baseline, or the line below it in its
/// stack, as `draw_fills` fills them.
fn fill_areas(line: &DrawnLine, lines: &[DrawnLine], scales: &Scales, chart_area: Rect) -> Vec<Vec<Position>> {
    let (_, value_id) = line.orientation.index_and_value_scales(&line.options);
    let Some(value_scale) = scales.get(value_id) else {
        return vec![];
    };

    let below = match (line.options.fill, line.below) {
        (Fill::Stack, Some(below)) => Some(&lines[below]),
        _ => None,
    };

    let mut areas = vec![];

    if let Some(polar_line) = &line.polar {
        if !polar_line.values_on_radius {
            return vec![];
        }

        let polar = &polar_line.polar;
        let baseline = match line.options.fill {
            Fill::Bottom => polar.inner_radius(),
            Fill::Top => polar.circle().radius,
            fill => {
                let value = fill.baseline_value().unwrap_or(0.0);
                let (low, high) = (value_scale.min().min(value_scale.max()), value_scale.min().max(value_scale.max()));
                polar.distance(value_scale.normalize(value.clamp(low, high)))
            }
        };
        let ring = Ring { polar, distance: baseline, corners: polar_line.corners.as_deref() };

        for run in &line.runs {
            if run.len() < 2 {
                continue;
            }

            let top: Vec<Position> = polar_line.points[run.clone()].iter().map(|point| point.position).collect();
            let (first, last) = (polar_line.points[run.start], polar_line.points[run.end - 1]);

            match (polar_line.closed, below.and_then(|below| below.polar.as_ref())) {
                (true, Some(below)) if below.closed => {
                    areas.push(top);
                    areas.push(below.points.iter().map(|point| point.position).collect());
                }
                (false, Some(below)) => {
                    let mut polygon = top;
                    polygon.extend(below.points.iter().rev().map(|point| point.position));
                    areas.push(polygon);
                }
                (true, _) => {
                    areas.push(top);
                    if baseline > 0.5 {
                        areas.push(ring_points(&ring, 0.0, std::f64::consts::TAU));
                    }
                }
                (false, _) => {
                    let mut polygon = top;
                    if baseline > 0.5 {
                        polygon.extend(ring_points(&ring, last.angle, first.angle));
                    } else {
                        polygon.push(polar.circle().center);
                    }
                    areas.push(polygon);
                }
            }
        }

        return areas;
    }

    for run in &line.runs {
        if run.len() < 2 {
            continue;
        }

        let points = &line.points[run.clone()];
        let mut polygon: Vec<Position> = points.to_vec();
        let (left, right) = (points[0].x.min(points[points.len() - 1].x), points[0].x.max(points[points.len() - 1].x));

        let lower: Vec<Position> = below.map(|below| {
            below.runs.iter().rev()
                .flat_map(|run| below.points[run.clone()].iter().rev())
                .filter(|point| point.x >= left - 0.5 && point.x <= right + 0.5)
                .copied()
                .collect()
        }).unwrap_or_default();

        if lower.is_empty() {
            let baseline = baseline_pixel(line.options.fill, value_scale, line.orientation.value_axis(), chart_area);
            polygon.push(Position::new(points[points.len() - 1].x, baseline));
            polygon.push(Position::new(points[0].x, baseline));
        } else {
            polygon.extend(lower);
        }

        areas.push(polygon.into_iter().map(|point| line.orientation.swap(point)).collect());
    }

    areas
}

/// Points along `ring` from screen angle `from` to `to`, close enough together for picking.
fn ring_points(ring: &Ring, from: Scalar, to: Scalar) -> Vec<Position> {
    const STEPS: usize = 48;
    (0..=STEPS).map(|step| ring.point(from + (to - from) * step as Scalar / STEPS as Scalar)).collect()
}

/// Registers each dataset's values (see `ScaleBounds::register_values`), its x on the scale its x
/// is placed on and its y on the scale its y is placed on, as `orientation` places them, and asks
/// for a polar dataset's scales' roles. Every
/// dataset reports, whether or not its points are drawn, as its marks are drawn on the same scales,
/// except a polar dataset when the controller doesn't draw around a circle (`polar` is false):
/// it is left out entirely, as the controller leaves it out of its bounds.
pub(crate) fn register_values<T, D>(datasets: &D, bounds: &mut ScaleBounds, orientation: Orientation, polar: bool)
where
    T: AsDataSet + ?Sized,
    D: Sequence<T>,
{
    datasets.foreach(&mut |_dataset_index, dataset| {
        let dataset = dataset.as_dataset();
        let options = dataset.options();
        if !polar && is_polar(&options, bounds.placement()) {
            return;
        }

        orientation.request_polar_roles(&options, bounds);
        let (index_scale, value_scale) = orientation.for_dataset(&options, bounds.placement()).index_and_value_scales(&options);
        let (index_scale, value_scale) = (index_scale.clone(), value_scale.clone());

        // The point's x goes on the index scale and its y on the value scale.
        bounds.register_values(dataset, &index_scale, &value_scale);
    });
}

/// Adds a legend entry for each of `datasets` that has a name: the entry the dataset's first mark
/// to give one asks for (`mark_entry`, which reads the marks with the context it is given), or
/// else the colour its line, area or bars are drawn in (see `DatasetColors::color`), with the
/// marker `marker` gives for its index and options.
pub(crate) fn legend_items<T: AsDataSet + ?Sized, D: Sequence<T>>(
    datasets: &D,
    colors: &SeriesColors,
    items: &mut Vec<LegendItem>,
    marker: impl Fn(usize, &DataSetOptions) -> LegendMarker,
    mark_entry: impl Fn(&T, &LegendContext) -> Option<(Color, LegendMarker)>,
) {
    datasets.foreach(&mut |dataset_index, dataset| {
        let options = dataset.as_dataset().options();

        let Some(label) = options.label.clone() else {
            return;
        };

        let context = LegendContext { colors: colors.dataset(options.color, dataset_index) };
        let (color, marker) = mark_entry(dataset, &context)
            .unwrap_or_else(|| (context.colors.color(), marker(dataset_index, &options)));

        items.push(LegendItem { label, color, marker, hidden: !options.visible });
    });
}

/// Describes the datum `key` of `datasets` (see `DatasetController::describe`): its dataset's
/// name, its index value as its index scale labels it, and its values (`DataPoint::describe`) as
/// its value scale labels them, the scales read as `orientation` reads them. Its colour and marker
/// are those `look` gives for the dataset, a mark's own colour or `None` for the point's colour.
/// `None` for a hidden dataset, or a datum it doesn't have.
pub(crate) fn describe_datum<T: AsDataSet + ?Sized, D: Sequence<T>>(
    datasets: &D,
    key: DatumKey,
    scales: &Scales,
    orientation: Orientation,
    colors: &SeriesColors,
    env: &mut Environment,
    look: impl Fn(&T, &DataSetOptions, usize) -> (Option<Color>, LegendMarker),
) -> Option<DatumInfo> {
    let mut found = None;
    datasets.foreach(&mut |dataset_index, item| {
        if dataset_index != key.dataset {
            return;
        }

        let dataset = item.as_dataset();
        let options = dataset.options();
        if !options.visible {
            return;
        }

        let (own_color, marker) = look(item, &options, dataset_index);
        let (index_id, value_id) = orientation.for_dataset(&options, scales).index_and_value_scales(&options);
        let coordinates = Coordinates::new(scales, index_id, value_id);

        dataset.points(&mut |point_index, point| {
            if point_index != key.index {
                return;
            }

            let mut values = vec![];
            point.describe(&mut |value| values.push(OwnedPointValue::from(value)));
            let color = own_color.unwrap_or_else(|| colors.dataset(options.color, dataset_index).point(point_index, point.color()));
            found = Some((options.label.clone(), color, marker, coordinates.x(point), values, index_id.clone(), value_id.clone()));
        });
    });

    let (dataset, color, marker, x, values, index_id, value_id) = found?;
    let index = match (x, scales.get(&index_id)) {
        (Some(x), Some(scale)) => Some(tick_label(scale, x, env)),
        _ => None,
    };
    let value_scale = scales.get(&value_id);
    let values = values.into_iter().map(|value| value.text(value_scale, env)).collect();

    Some(DatumInfo { key, dataset, color, marker, index, values })
}

/// A point's value (`PointValue`), kept past the point it was read from.
enum OwnedPointValue {
    Value(Option<&'static str>, OwnedValue),
    Span(Option<&'static str>, OwnedValue, OwnedValue),
    Other(&'static str, OwnedValue),
}

/// A `Value`, kept past the point it was read from.
enum OwnedValue {
    Number(Scalar),
    Text(String),
}

impl From<Value<'_>> for OwnedValue {
    fn from(value: Value) -> OwnedValue {
        match value {
            Value::Number(number) | Value::Time(number) => OwnedValue::Number(number),
            Value::Text(text) => OwnedValue::Text(text.to_string()),
        }
    }
}

impl From<PointValue<'_>> for OwnedPointValue {
    fn from(value: PointValue) -> OwnedPointValue {
        match value {
            PointValue::Value { name, value } => OwnedPointValue::Value(name, value.into()),
            PointValue::Span { name, low, high } => OwnedPointValue::Span(name, low.into(), high.into()),
            PointValue::Other { name, value } => OwnedPointValue::Other(name, value.into()),
        }
    }
}

impl OwnedValue {
    /// The value as `scale` labels its ticks, or as a plain number without a scale.
    fn text(self, scale: Option<&dyn Scale>, env: &mut Environment) -> String {
        match (self, scale) {
            (OwnedValue::Text(text), _) => text,
            (OwnedValue::Number(number), Some(scale)) => tick_label(scale, number, env),
            (OwnedValue::Number(number), None) => format_number(number, env),
        }
    }
}

impl OwnedPointValue {
    /// The value as text, along `value_scale` unless it isn't a value-scale value.
    fn text(self, value_scale: Option<&dyn Scale>, env: &mut Environment) -> ValueText {
        match self {
            OwnedPointValue::Value(name, value) => ValueText { name: name.map(str::to_string), text: value.text(value_scale, env) },
            OwnedPointValue::Span(name, low, high) => {
                let text = format!("{} – {}", low.text(value_scale, env), high.text(value_scale, env));
                ValueText { name: name.map(str::to_string), text }
            }
            OwnedPointValue::Other(name, value) => ValueText { name: Some(name.to_string()), text: value.text(None, env) },
        }
    }
}

/// Shows or hides the dataset behind legend entry `entry`, counting entries as `legend_items`
/// adds them, one per named dataset. Returns whether the dataset could be shown or hidden (see
/// `DataSet::set_visible`).
pub(crate) fn set_legend_visibility<T: AsDataSet + ?Sized, D: Sequence<T>>(datasets: &D, entry: usize, visible: bool) -> bool {
    let mut named = 0;
    let mut set = false;
    datasets.foreach(&mut |_dataset_index, dataset| {
        let dataset = dataset.as_dataset();
        if dataset.options().label.is_none() {
            return;
        }
        if named == entry {
            set = dataset.set_visible(visible);
        }
        named += 1;
    });
    set
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
    use crate::controller::orientation::Orientation;
    use super::{baseline_pixel, drawn_lines};

    const SALES: StackId = StackId::new("sales");

    fn y_range(controller: &dyn DatasetController) -> Option<(f64, f64)> {
        let mut bounds = ScaleBounds::default();
        controller.collect_bounds(&mut bounds, &mut Environment::new());
        bounds.requests().get(&ScaleId::Y).range()
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
        let lines = drawn_lines::<dyn LineDataSet, _>(&controller.dataset_sequence, &scales, area, true, true, Orientation::Vertical, |_| true);

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
        let lines = drawn_lines::<dyn LineDataSet, _>(&controller.dataset_sequence, &scales, area, true, true, Orientation::Vertical, |_| true);

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

    /// On a zoomed scale, only the points in the window are laid out, with one either side; when
    /// none is inside, the two whose line crosses it; and all of them when they aren't in order.
    #[test]
    fn zoomed_lines_keep_the_points_in_the_window_and_one_either_side() {
        use super::visible_points;

        let rising = [(0, -0.5), (1, -0.2), (2, 0.1), (3, 0.6), (4, 1.2), (5, 1.8)];
        assert_eq!(visible_points(&rising), Some(1..=4));

        let falling = [(0, 1.5), (1, 0.9), (2, 0.4), (3, -0.3)];
        assert_eq!(visible_points(&falling), Some(0..=3));

        let across = [(0, -0.5), (1, -0.1), (3, 1.4)];
        assert_eq!(visible_points(&across), Some(1..=3), "no point inside, but the line from 1 to 3 crosses the window");

        let aside = [(0, 1.2), (1, 1.5)];
        assert!(visible_points(&aside).is_some_and(|kept| kept.is_empty()), "nothing near the window");

        let scattered = [(0, 0.5), (1, -2.0), (2, 0.7)];
        assert_eq!(visible_points(&scattered), None, "out of order, so all of them");
    }
}
