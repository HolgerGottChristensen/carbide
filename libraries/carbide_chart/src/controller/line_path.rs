//! The path of a line through a dataset's points, for each `Interpolation`. Points are in pixels,
//! already mapped through the dataset's scales: a tension spline depends on distances, which
//! only mean something once both axes are in the same units.

use carbide::draw::{Position, Rect, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::element::{Interpolation, Stepped};

/// Adds the line through `points` to the current path, starting with a `move_to` at the first
/// point. Curves are kept inside `area` (see `cap_control_points`).
pub(crate) fn trace_line(ctx: &mut CanvasContext, points: &[Position], interpolation: Interpolation, area: Rect) {
    let Some(first) = points.first() else {
        return;
    };

    ctx.move_to(first.x, first.y);

    match interpolation {
        Interpolation::Linear => trace_straight(ctx, points),
        // A tension of 0 is a straight line.
        Interpolation::Smooth { tension: 0.0 } => trace_straight(ctx, points),
        Interpolation::Stepped(stepped) => {
            for pair in points.windows(2) {
                trace_step(ctx, pair[0], pair[1], stepped);
            }
        }
        Interpolation::Smooth { tension } => {
            let mut controls = spline_control_points(points, tension);
            cap_control_points(points, &mut controls, area);
            trace_curves(ctx, points, &controls);
        }
        Interpolation::Monotone => {
            let mut controls = monotone_control_points(points);
            cap_control_points(points, &mut controls, area);
            trace_curves(ctx, points, &controls);
        }
    }
}

/// Straight lines from each point to the next: the path is already at the first point.
fn trace_straight(ctx: &mut CanvasContext, points: &[Position]) {
    for point in &points[1..] {
        ctx.line_to(point.x, point.y);
    }
}

/// The step from `from` to `to`: the path is already at `from`.
fn trace_step(ctx: &mut CanvasContext, from: Position, to: Position, stepped: Stepped) {
    match stepped {
        Stepped::Before => {
            ctx.line_to(from.x, to.y);
            ctx.line_to(to.x, to.y);
        }
        Stepped::After => {
            ctx.line_to(to.x, from.y);
            ctx.line_to(to.x, to.y);
        }
        Stepped::Middle => {
            let middle_x = (to.x - from.x) / 2.0 + from.x;
            ctx.line_to(middle_x, from.y);
            ctx.line_to(middle_x, to.y);
            ctx.line_to(to.x, to.y);
        }
        Stepped::MiddleVertical => {
            let middle_y = (to.y - from.y) / 2.0 + from.y;
            ctx.line_to(from.x, middle_y);
            ctx.line_to(to.x, middle_y);
            ctx.line_to(to.x, to.y);
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
fn trace_curves(ctx: &mut CanvasContext, points: &[Position], controls: &[ControlPoints]) {
    for index in 1..points.len() {
        ctx.bezier_curve_to(controls[index - 1].after, controls[index].before, points[index]);
    }
}

/// Control points for a tension spline through `points`: at each point, the tangent is parallel to
/// the line from the previous point to the next, and its two arms are `tension` times that line's
/// length, split in proportion to the distances to the two neighbours. The end points use
/// themselves as their missing neighbour.
fn spline_control_points(points: &[Position], tension: Scalar) -> Vec<ControlPoints> {
    let last = points.len().saturating_sub(1);

    (0..points.len())
        .map(|index| {
            let previous = points[index.saturating_sub(1)];
            let current = points[index];
            let next = points[(index + 1).min(last)];

            let d01 = distance(previous, current);
            let d12 = distance(current, next);

            // All three at the same place: no tangent, so both control points are the point.
            let (s01, s12) = if d01 + d12 > 0.0 {
                (d01 / (d01 + d12), d12 / (d01 + d12))
            } else {
                (0.0, 0.0)
            };

            let fa = tension * s01;
            let fb = tension * s12;
            let dx = next.x - previous.x;
            let dy = next.y - previous.y;

            ControlPoints {
                before: Position::new(current.x - fa * dx, current.y - fa * dy),
                after: Position::new(current.x + fb * dx, current.y + fb * dy),
            }
        })
        .collect()
}

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
    // `Rect`'s top and bottom are swapped on screen (see `Scale::draw_grid`), so take both ends.
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
    use carbide::draw::{Dimension, Position, Rect};
    use super::{cap_control_points, monotone_control_points, spline_control_points};

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
        let spline = spline_control_points(&points, 0.4);
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
        let controls = spline_control_points(&points, 0.5);

        // Equal distances to both neighbours: each arm is 0.5 * 0.5 of the 20px from 0 to 20.
        assert_eq!(controls[1].before, Position::new(5.0, 10.0));
        assert_eq!(controls[1].after, Position::new(15.0, 10.0));

        // The ends use themselves as the missing neighbour.
        assert_eq!(controls[0].before, points[0]);
    }

    /// Control points between two points inside the area are pulled into it; a single point or
    /// several at the same place don't produce NaNs.
    #[test]
    fn control_points_are_capped_to_the_area() {
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(100.0, 100.0));
        let points = positions(&[(0.0, 50.0), (50.0, 100.0), (100.0, 50.0)]);
        let mut controls = spline_control_points(&points, 1.0);
        controls[1].before.y = 130.0;
        controls[1].after.y = 130.0;

        cap_control_points(&points, &mut controls, area);
        assert_eq!(controls[1].before.y, 100.0);
        assert_eq!(controls[1].after.y, 100.0);

        let same = positions(&[(5.0, 5.0), (5.0, 5.0)]);
        assert!(spline_control_points(&same, 0.4).iter().all(|c| c.before.x.is_finite() && c.after.y.is_finite()));
        assert!(monotone_control_points(&same).iter().all(|c| c.before.x.is_finite() && c.after.y.is_finite()));
        assert_eq!(monotone_control_points(&positions(&[(1.0, 2.0)])).len(), 1);
    }
}
