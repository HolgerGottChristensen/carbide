//! The path of a line through a dataset's points, for each `Interpolation`. Points are in pixels,
//! already mapped through the dataset's scales: a tension spline depends on distances, which
//! only mean something once both axes are in the same units.
//!
//! The line is made of one segment between each pair of neighbouring points (`Segment`), which
//! can be traced forwards, for the line itself, or backwards, for the lower edge of a stacked
//! area, which runs back along the line below it.

use carbide::draw::{Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::element::{Interpolation, Stepped};

/// Where a path is traced to: the canvas when drawing, or a list of commands in tests.
pub(crate) trait PathSink {
    fn move_to(&mut self, to: Position);
    fn line_to(&mut self, to: Position);
    fn bezier_curve_to(&mut self, control1: Position, control2: Position, to: Position);
}

impl PathSink for CanvasContext<'_, '_, '_> {
    fn move_to(&mut self, to: Position) {
        CanvasContext::move_to(self, to.x, to.y);
    }

    fn line_to(&mut self, to: Position) {
        CanvasContext::line_to(self, to.x, to.y);
    }

    fn bezier_curve_to(&mut self, control1: Position, control2: Position, to: Position) {
        CanvasContext::bezier_curve_to(self, control1, control2, to);
    }
}

/// Adds the line through `points` to the current path, starting with a `move_to` at the first
/// point. Curves are kept inside `area` (see `cap_control_points`).
pub(crate) fn trace_line(path: &mut impl PathSink, points: &[Position], interpolation: Interpolation, area: Rect) {
    let Some(first) = points.first() else {
        return;
    };

    path.move_to(*first);

    for (index, segment) in segments(points, interpolation, area).iter().enumerate() {
        segment.trace(path, points[index + 1]);
    }
}

/// Adds the line through `points` to the current path backwards, from the last point to the
/// first, along exactly the same path `trace_line` draws. It continues the current path with a
/// `line_to` the last point, so it can close an area whose top was traced forwards.
pub(crate) fn trace_line_reversed(path: &mut impl PathSink, points: &[Position], interpolation: Interpolation, area: Rect) {
    let Some(last) = points.last() else {
        return;
    };

    path.line_to(*last);

    for (index, segment) in segments(points, interpolation, area).iter().enumerate().rev() {
        segment.trace_reversed(path, points[index]);
    }
}

/// The path between two neighbouring points, without its end points.
#[derive(Clone, Debug, PartialEq)]
enum Segment {
    Straight,
    /// The corners of a step, in order from the first point to the second.
    Corners(Vec<Position>),
    /// A cubic Bézier curve, with its control points in order from the first point.
    Curve(Position, Position),
}

impl Segment {
    /// From the segment's first point, where the path is, to `to`.
    fn trace(&self, path: &mut impl PathSink, to: Position) {
        match self {
            Segment::Straight => path.line_to(to),
            Segment::Corners(corners) => {
                for corner in corners {
                    path.line_to(*corner);
                }
                path.line_to(to);
            }
            Segment::Curve(control1, control2) => path.bezier_curve_to(*control1, *control2, to),
        }
    }

    /// From the segment's second point, where the path is, back to `to`, its first.
    fn trace_reversed(&self, path: &mut impl PathSink, to: Position) {
        match self {
            Segment::Straight => path.line_to(to),
            Segment::Corners(corners) => {
                for corner in corners.iter().rev() {
                    path.line_to(*corner);
                }
                path.line_to(to);
            }
            Segment::Curve(control1, control2) => path.bezier_curve_to(*control2, *control1, to),
        }
    }
}

/// The segments between each pair of neighbouring points, one fewer than the points.
fn segments(points: &[Position], interpolation: Interpolation, area: Rect) -> Vec<Segment> {
    let pairs = points.windows(2);

    match interpolation {
        Interpolation::Linear => pairs.map(|_| Segment::Straight).collect(),
        // A tension of 0 is a straight line.
        Interpolation::Smooth { tension: 0.0 } => pairs.map(|_| Segment::Straight).collect(),
        Interpolation::Stepped(stepped) => pairs.map(|pair| Segment::Corners(step_corners(pair[0], pair[1], stepped))).collect(),
        Interpolation::Smooth { .. } | Interpolation::Cardinal { .. } | Interpolation::CatmullRom { .. } => {
            let mut controls = spline_control_points(points, interpolation);
            cap_control_points(points, &mut controls, area);
            curves(&controls)
        }
        Interpolation::Monotone => {
            let mut controls = monotone_control_points(points);
            cap_control_points(points, &mut controls, area);
            curves(&controls)
        }
    }
}

/// The corners of the step from `from` to `to`.
fn step_corners(from: Position, to: Position, stepped: Stepped) -> Vec<Position> {
    match stepped {
        Stepped::Before => vec![Position::new(from.x, to.y)],
        Stepped::After => vec![Position::new(to.x, from.y)],
        Stepped::Middle => {
            let middle_x = (to.x - from.x) / 2.0 + from.x;
            vec![Position::new(middle_x, from.y), Position::new(middle_x, to.y)]
        }
        Stepped::MiddleVertical => {
            let middle_y = (to.y - from.y) / 2.0 + from.y;
            vec![Position::new(from.x, middle_y), Position::new(to.x, middle_y)]
        }
    }
}

/// The two Bézier control points around one point of the line: `before` shapes the curve
/// arriving at the point, `after` the curve leaving it.
#[derive(Copy, Clone, Debug, PartialEq)]
struct ControlPoints {
    before: Position,
    after: Position,
}

/// One cubic Bézier curve per pair of neighbouring points, from the first point's `after` control
/// point to the second point's `before`.
fn curves(controls: &[ControlPoints]) -> Vec<Segment> {
    controls.windows(2).map(|pair| Segment::Curve(pair[0].after, pair[1].before)).collect()
}

/// Control points for the spline `interpolation` draws through `points` (see
/// `spline_controls_at`). The end points use themselves as their missing neighbour.
fn spline_control_points(points: &[Position], interpolation: Interpolation) -> Vec<ControlPoints> {
    let last = points.len().saturating_sub(1);

    (0..points.len())
        .map(|index| {
            let previous = points[index.saturating_sub(1)];
            let next = points[(index + 1).min(last)];
            let (before, after) = spline_controls_at(interpolation, previous, points[index], next);
            ControlPoints { before, after }
        })
        .collect()
}

/// The control points before and after `current` on the spline `interpolation` draws, from its
/// neighbours `previous` and `next`. Only the splines whose control points depend on the
/// neighbours alone are drawn this way: `Smooth`, `Cardinal` and `CatmullRom`; any other
/// interpolation gets the point itself on both sides.
///
/// - `Smooth`: the tangent is parallel to the line from `previous` to `next`, and its two arms
///   are `tension` times that line's length, split in proportion to the distances to the two
///   neighbours.
/// - `Cardinal`: the same tangent, with arms of a sixth of that line on both sides, shortened by
///   `tension`.
/// - `CatmullRom`: the Bézier form of the Catmull-Rom spline with the distances to the
///   neighbours raised to `alpha` (Barry and Goldman's parametrisation). A neighbour at the
///   same place as `current` gives the point itself on that side.
pub(crate) fn spline_controls_at(interpolation: Interpolation, previous: Position, current: Position, next: Position) -> (Position, Position) {
    match interpolation {
        Interpolation::Smooth { tension } => {
            let d01 = distance(previous, current);
            let d12 = distance(current, next);

            // All three at the same place: no tangent, so both control points are the point.
            let (s01, s12) = if d01 + d12 > 0.0 {
                (d01 / (d01 + d12), d12 / (d01 + d12))
            } else {
                (0.0, 0.0)
            };

            let (fa, fb) = (tension * s01, tension * s12);
            let (dx, dy) = (next.x - previous.x, next.y - previous.y);

            (
                Position::new(current.x - fa * dx, current.y - fa * dy),
                Position::new(current.x + fb * dx, current.y + fb * dy),
            )
        }
        Interpolation::Cardinal { tension } => {
            let arm = (1.0 - tension) / 6.0;
            let (dx, dy) = (arm * (next.x - previous.x), arm * (next.y - previous.y));

            (Position::new(current.x - dx, current.y - dy), Position::new(current.x + dx, current.y + dy))
        }
        Interpolation::CatmullRom { alpha } => {
            // The distances to the neighbours, raised to alpha and to twice alpha.
            let (to_previous, to_next) = (distance(previous, current), distance(current, next));
            let (lp, ln) = (to_previous.powf(alpha), to_next.powf(alpha));
            let (lp2, ln2) = (to_previous.powf(2.0 * alpha), to_next.powf(2.0 * alpha));

            let before = if ln > EPSILON {
                let (weight, divisor) = (2.0 * ln2 + 3.0 * ln * lp + lp2, 3.0 * ln * (ln + lp));
                Position::new(
                    (current.x * weight + previous.x * ln2 - next.x * lp2) / divisor,
                    (current.y * weight + previous.y * ln2 - next.y * lp2) / divisor,
                )
            } else {
                current
            };

            let after = if lp > EPSILON {
                let (weight, divisor) = (2.0 * lp2 + 3.0 * lp * ln + ln2, 3.0 * lp * (lp + ln));
                Position::new(
                    (current.x * weight - previous.x * ln2 + next.x * lp2) / divisor,
                    (current.y * weight - previous.y * ln2 + next.y * lp2) / divisor,
                )
            } else {
                current
            };

            (before, after)
        }
        _ => (current, current),
    }
}

/// Distances, raised to a Catmull-Rom spline's alpha, below which a neighbour is at the same
/// place as the point.
const EPSILON: Scalar = 1e-12;

/// Control points for a monotone cubic through `points`, which are in x order: the curve never
/// goes above or below both of a segment's end points, so it adds no peaks the data doesn't
/// have. Fritsch–Carlson monotone cubic interpolation.
fn monotone_control_points(points: &[Position]) -> Vec<ControlPoints> {
    let count = points.len();
    if count == 0 {
        return vec![];
    }

    // The slope of each segment; 0 for two points at the same x.
    let mut slopes = vec![0.0; count];
    for index in 0..count - 1 {
        let dx = points[index + 1].x - points[index].x;
        slopes[index] = if dx != 0.0 { (points[index + 1].y - points[index].y) / dx } else { 0.0 };
    }

    // The tangent at each point: the mean of the slopes on either side, flat where the
    // data turns, and the neighbouring slope at the ends.
    let mut tangents: Vec<Scalar> = (0..count)
        .map(|index| {
            if index == 0 {
                slopes[0]
            } else if index == count - 1 {
                slopes[index - 1]
            } else if sign(slopes[index - 1]) != sign(slopes[index]) {
                0.0
            } else {
                (slopes[index - 1] + slopes[index]) / 2.0
            }
        })
        .collect();

    // Shrinks tangents that would make a segment overshoot: flat segments get flat tangents, and
    // tangents too steep for their segment are scaled back onto a circle of radius 3.
    for index in 0..count - 1 {
        if slopes[index].abs() < Scalar::EPSILON {
            tangents[index] = 0.0;
            tangents[index + 1] = 0.0;
            continue;
        }

        let alpha = tangents[index] / slopes[index];
        let beta = tangents[index + 1] / slopes[index];
        let squared_magnitude = alpha * alpha + beta * beta;

        if squared_magnitude <= 9.0 {
            continue;
        }

        let tau = 3.0 / squared_magnitude.sqrt();
        tangents[index] = alpha * tau * slopes[index];
        tangents[index + 1] = beta * tau * slopes[index];
    }

    // Each control point is a third of the way along x to its neighbour, on the tangent.
    (0..count)
        .map(|index| {
            let point = points[index];
            let tangent = tangents[index];

            let before = if index > 0 {
                let delta = (point.x - points[index - 1].x) / 3.0;
                Position::new(point.x - delta, point.y - delta * tangent)
            } else {
                point
            };

            let after = if index < count - 1 {
                let delta = (points[index + 1].x - point.x) / 3.0;
                Position::new(point.x + delta, point.y + delta * tangent)
            } else {
                point
            };

            ControlPoints { before, after }
        })
        .collect()
}

/// Keeps the control points between two points inside `area`, so a curve between points inside
/// the chart area doesn't bulge out of it. Control points next to a point outside the area are
/// left alone.
fn cap_control_points(points: &[Position], controls: &mut [ControlPoints], area: Rect) {
    // `Rect`'s top and bottom are swapped on screen (see `scale/axis.rs`), so take both ends.
    let (left, right) = (area.left().min(area.right()), area.left().max(area.right()));
    let (top, bottom) = (area.top().min(area.bottom()), area.top().max(area.bottom()));

    // A half-pixel margin, so points on the area's edge count as inside despite rounding.
    let in_area = |point: Position| {
        point.x > left - 0.5 && point.x < right + 0.5 && point.y > top - 0.5 && point.y < bottom + 0.5
    };
    let cap = |point: Position| Position::new(point.x.clamp(left, right), point.y.clamp(top, bottom));

    for index in 0..points.len() {
        if !in_area(points[index]) {
            continue;
        }

        if index > 0 && in_area(points[index - 1]) {
            controls[index].before = cap(controls[index].before);
        }

        if index + 1 < points.len() && in_area(points[index + 1]) {
            controls[index].after = cap(controls[index].after);
        }
    }
}

fn distance(a: Position, b: Position) -> Scalar {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

/// -1, 0 or 1. Unlike `f64::signum`, zero has a sign of its own, so a flat slope never counts
/// as rising.
fn sign(value: Scalar) -> i8 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Dimension, Position, Rect, Scalar};
    use crate::element::{Interpolation, Stepped};
    use super::{cap_control_points, distance, monotone_control_points, spline_control_points, trace_line, trace_line_reversed, PathSink};

    /// The commands a path is traced with.
    #[derive(Debug, PartialEq)]
    enum Command {
        Move(Position),
        Line(Position),
        Curve(Position, Position, Position),
    }

    impl PathSink for Vec<Command> {
        fn move_to(&mut self, to: Position) {
            self.push(Command::Move(to));
        }

        fn line_to(&mut self, to: Position) {
            self.push(Command::Line(to));
        }

        fn bezier_curve_to(&mut self, control1: Position, control2: Position, to: Position) {
            self.push(Command::Curve(control1, control2, to));
        }
    }

    /// Traced in reverse, a line runs back along exactly the path it is drawn with: the same
    /// points and step corners in the opposite order, and each curve with its control points
    /// swapped, for every interpolation.
    #[test]
    fn reversed_lines_follow_the_same_path_backwards() {
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(100.0, 100.0));
        let points = positions(&[(0.0, 50.0), (20.0, 80.0), (45.0, 30.0), (70.0, 60.0), (100.0, 40.0)]);

        for interpolation in [
            Interpolation::Linear,
            Interpolation::Smooth { tension: 0.4 },
            Interpolation::Cardinal { tension: 0.3 },
            Interpolation::CatmullRom { alpha: 0.5 },
            Interpolation::Monotone,
            Interpolation::Stepped(Stepped::Before),
            Interpolation::Stepped(Stepped::Middle),
            Interpolation::Stepped(Stepped::MiddleVertical),
        ] {
            let mut forward = vec![];
            trace_line(&mut forward, &points, interpolation, area);
            let mut reversed = vec![];
            trace_line_reversed(&mut reversed, &points, interpolation, area);

            // Where each command ends, starting with the forward `move_to` and the reversed
            // `line_to` the last point.
            let ends = |commands: &[Command]| -> Vec<Position> {
                commands.iter().map(|command| match command {
                    Command::Move(to) | Command::Line(to) | Command::Curve(_, _, to) => *to,
                }).collect()
            };
            let mut forward_ends = ends(&forward);
            forward_ends.reverse();
            assert_eq!(ends(&reversed), forward_ends, "{interpolation:?}: the reversed path should visit the same points backwards");

            let curves = |commands: &[Command]| -> Vec<(Position, Position)> {
                commands.iter().filter_map(|command| match command {
                    Command::Curve(control1, control2, _) => Some((*control1, *control2)),
                    _ => None,
                }).collect()
            };
            let swapped: Vec<(Position, Position)> = curves(&forward).into_iter().rev().map(|(control1, control2)| (control2, control1)).collect();
            assert_eq!(curves(&reversed), swapped, "{interpolation:?}: each curve should be traced with its control points swapped");
        }
    }

    fn positions(points: &[(f64, f64)]) -> Vec<Position> {
        points.iter().map(|&(x, y)| Position::new(x, y)).collect()
    }

    /// Between two points, the monotone curve's control points stay within the two points'
    /// y range, so the curve can't go above or below both: rising data gives a rising line,
    /// even across a steep jump next to a nearly flat stretch, where a tension spline bulges.
    #[test]
    fn monotone_curves_do_not_overshoot() {
        let points = positions(&[(0.0, 0.0), (10.0, 10.0), (20.0, 11.0), (30.0, 50.0), (40.0, 51.0)]);
        let controls = monotone_control_points(&points);

        for index in 1..points.len() {
            let (low, high) = (points[index - 1].y.min(points[index].y), points[index - 1].y.max(points[index].y));

            for control in [controls[index - 1].after, controls[index].before] {
                assert!(low - 1e-9 <= control.y && control.y <= high + 1e-9, "segment {index}: control point y {} is outside {low}..{high}", control.y);
            }
        }

        // On the same data, the tension spline dips below the segment from 10 to 11 before the
        // jump, so this data does tell the two apart.
        let spline = spline_control_points(&points, Interpolation::Smooth { tension: 0.4 });
        assert!(spline[2].before.y < points[1].y, "the tension spline should overshoot here, at {}", spline[2].before.y);
    }

    /// A peak in the data is a peak of the monotone curve: the tangent there is flat.
    #[test]
    fn monotone_peaks_are_flat() {
        let points = positions(&[(0.0, 0.0), (10.0, 30.0), (20.0, 5.0)]);
        let controls = monotone_control_points(&points);

        assert_eq!(controls[1].before.y, 30.0);
        assert_eq!(controls[1].after.y, 30.0);
    }

    /// A flat segment stays flat: the tangents at both of its ends are flat, also where a rising
    /// segment follows.
    #[test]
    fn monotone_flat_segments_stay_flat() {
        let points = positions(&[(0.0, 0.0), (10.0, 0.0), (20.0, 10.0)]);
        let controls = monotone_control_points(&points);

        assert_eq!(controls[0].after.y, 0.0);
        assert_eq!(controls[1].before.y, 0.0);
        assert_eq!(controls[1].after.y, 0.0, "the tangent at the turn should be flat");
    }

    /// The spline's tangent at a point is parallel to the line from its previous point to its
    /// next, with arms `tension` times that line, split by the distances to each neighbour.
    #[test]
    fn spline_tangents_follow_the_neighbours() {
        let points = positions(&[(0.0, 0.0), (10.0, 10.0), (20.0, 0.0)]);
        let controls = spline_control_points(&points, Interpolation::Smooth { tension: 0.5 });

        // Equal distances to both neighbours: each arm is 0.5 * 0.5 of the 20px from 0 to 20.
        assert_eq!(controls[1].before, Position::new(5.0, 10.0));
        assert_eq!(controls[1].after, Position::new(15.0, 10.0));

        // The ends use themselves as the missing neighbour.
        assert_eq!(controls[0].before, points[0]);
    }

    fn close(a: Position, b: Position) -> bool {
        (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9
    }

    /// A cardinal spline's arms are a sixth of the line from the previous point to the next on
    /// both sides, shortened by the tension, and a tension of 1 puts the control points on the
    /// points, so the curves are straight.
    #[test]
    fn cardinal_arms_are_even_and_shortened_by_the_tension() {
        let points = positions(&[(0.0, 0.0), (10.0, 10.0), (40.0, 0.0)]);

        let loose = spline_control_points(&points, Interpolation::Cardinal { tension: 0.0 });
        assert!(close(loose[1].before, Position::new(10.0 - 40.0 / 6.0, 10.0)));
        assert!(close(loose[1].after, Position::new(10.0 + 40.0 / 6.0, 10.0)));

        let half = spline_control_points(&points, Interpolation::Cardinal { tension: 0.5 });
        assert!(close(half[1].after, Position::new(10.0 + 20.0 / 6.0, 10.0)));

        let tight = spline_control_points(&points, Interpolation::Cardinal { tension: 1.0 });
        assert!(tight.iter().zip(&points).all(|(controls, point)| controls.before == *point && controls.after == *point));
    }

    /// A Catmull-Rom spline with an alpha of 0 is the uniform spline: the same curve as a
    /// cardinal spline with a tension of 0, the ends included.
    #[test]
    fn uniform_catmull_rom_is_a_cardinal_spline_without_tension() {
        let points = positions(&[(0.0, 50.0), (20.0, 80.0), (45.0, 30.0), (70.0, 60.0), (100.0, 40.0)]);

        let uniform = spline_control_points(&points, Interpolation::CatmullRom { alpha: 0.0 });
        let cardinal = spline_control_points(&points, Interpolation::Cardinal { tension: 0.0 });

        for (uniform, cardinal) in uniform.iter().zip(&cardinal) {
            assert!(close(uniform.before, cardinal.before) && close(uniform.after, cardinal.after), "{uniform:?} and {cardinal:?}");
        }
    }

    /// On evenly spaced points the distances are all the same, so every alpha draws the uniform
    /// spline; with a near neighbour on one side and a far one on the other, the centripetal and
    /// chordal splines shorten the arm towards the near one, where the uniform spline would
    /// overshoot it.
    #[test]
    fn catmull_rom_arms_follow_the_distances() {
        let even = positions(&[(0.0, 0.0), (10.0, 10.0), (20.0, 0.0), (30.0, 10.0)]);
        let uniform = spline_control_points(&even, Interpolation::CatmullRom { alpha: 0.0 });
        for alpha in [0.5, 1.0] {
            let spline = spline_control_points(&even, Interpolation::CatmullRom { alpha });
            assert!(close(spline[1].before, uniform[1].before) && close(spline[1].after, uniform[1].after), "alpha {alpha}");
        }

        // Far from the previous point, near the next.
        let uneven = positions(&[(0.0, 0.0), (100.0, 0.0), (102.0, 2.0)]);
        let arm = |alpha: Scalar| {
            let controls = spline_control_points(&uneven, Interpolation::CatmullRom { alpha });
            distance(uneven[1], controls[1].after)
        };
        assert!(arm(0.5) < arm(0.0), "the centripetal arm, {}, should be shorter than the uniform one, {}", arm(0.5), arm(0.0));
        assert!(arm(1.0) < arm(0.5), "the chordal arm, {}, should be shorter still than the centripetal one, {}", arm(1.0), arm(0.5));
        // The segment to the near point is under 3px long; the centripetal arm stays within it.
        assert!(arm(0.5) < distance(uneven[1], uneven[2]));
    }

    /// Control points between two points inside the area are pulled into it; a single point or
    /// several at the same place don't produce NaNs.
    #[test]
    fn control_points_are_capped_to_the_area() {
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(100.0, 100.0));
        let points = positions(&[(0.0, 50.0), (50.0, 100.0), (100.0, 50.0)]);
        let mut controls = spline_control_points(&points, Interpolation::Smooth { tension: 1.0 });
        controls[1].before.y = 130.0;
        controls[1].after.y = 130.0;

        cap_control_points(&points, &mut controls, area);
        assert_eq!(controls[1].before.y, 100.0);
        assert_eq!(controls[1].after.y, 100.0);

        let same = positions(&[(5.0, 5.0), (5.0, 5.0)]);
        assert!(spline_control_points(&same, Interpolation::Smooth { tension: 0.4 }).iter().all(|c| c.before.x.is_finite() && c.after.y.is_finite()));
        assert!(monotone_control_points(&same).iter().all(|c| c.before.x.is_finite() && c.after.y.is_finite()));
        assert_eq!(monotone_control_points(&positions(&[(1.0, 2.0)])).len(), 1);
    }

    /// A smooth line with a tension of 0 is a straight line: it is traced with the same straight
    /// segments as `Linear`, not with curves whose control points sit on the points.
    #[test]
    fn smooth_lines_without_tension_are_straight() {
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(100.0, 100.0));
        let points = positions(&[(0.0, 50.0), (20.0, 80.0), (45.0, 30.0), (100.0, 40.0)]);

        let mut linear = vec![];
        trace_line(&mut linear, &points, Interpolation::Linear, area);
        let mut flat = vec![];
        trace_line(&mut flat, &points, Interpolation::Smooth { tension: 0.0 }, area);

        assert_eq!(flat, linear);
        assert!(flat.iter().all(|command| !matches!(command, Command::Curve(..))));
    }
}
