//! The path of a line through a polar dataset's points. Points are mapped to the screen first and
//! then traced, so a straight segment is a chord between two spokes, the radar convention, and a
//! smooth line is a spline through the screen positions. Steps follow the grid: around a ring at
//! one point's distance from the centre, an arc when the rings are circles and straight from
//! corner to corner when they are polygons, and along a spoke to the next.
//!
//! On a full circle a line can close into a loop: the last point joins the first, through the
//! same interpolation, and a smooth loop has no ends, so every point's tangent comes from its
//! neighbours on both sides.

use carbide::draw::{Position, Scalar};
use crate::controller::line_path::{spline_controls_at, PathSink};
use crate::element::{Interpolation, Stepped};
use crate::scale::polar::RingShape;

/// The tension of the spline `Interpolation::Monotone` draws as around a circle, which has no
/// direction to be monotone in.
const LOOP_TENSION: Scalar = 0.4;

/// A point of a polar line: where it is on the screen, and its screen angle (radians clockwise
/// from the top) and distance from the centre, which steps follow.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct PolarPoint {
    pub(crate) position: Position,
    pub(crate) angle: Scalar,
    pub(crate) distance: Scalar,
}

/// Adds the line through `points` to the current path, starting with a `move_to` at the first
/// point, its steps around the rings of `rings`. With `closing`, the line goes on from the last point to it: the first
/// point again, a full turn on, so steps keep going the same way round. `Interpolation::Smooth`
/// with `closing` is a closed spline.
pub(crate) fn trace_polar_line(path: &mut (impl PathSink + ?Sized), points: &[PolarPoint], closing: Option<PolarPoint>, interpolation: Interpolation, rings: RingShape) {
    let Some(first) = points.first() else {
        return;
    };

    path.move_to(first.position);
    trace_from_first(path, points, closing, interpolation, rings);
}

/// Adds the line through `points` to the current path backwards, from the last point to the
/// first, continuing the path with a `line_to` the last point: the lower edge of a stacked area
/// whose top was traced forwards. Steps turn the other way round, so the edge follows the same
/// path the line itself is drawn along.
pub(crate) fn trace_polar_line_reversed(path: &mut (impl PathSink + ?Sized), points: &[PolarPoint], interpolation: Interpolation, rings: RingShape) {
    let Some(last) = points.last() else {
        return;
    };

    let reversed: Vec<PolarPoint> = points.iter().rev().copied().collect();
    let interpolation = match interpolation {
        Interpolation::Stepped(Stepped::Before) => Interpolation::Stepped(Stepped::After),
        Interpolation::Stepped(Stepped::After) => Interpolation::Stepped(Stepped::Before),
        interpolation => interpolation,
    };

    path.line_to(last.position);
    trace_from_first(path, &reversed, None, interpolation, rings);
}

/// Traces from the first of `points`, where the path is, to the last, then on to `closing`.
fn trace_from_first(path: &mut (impl PathSink + ?Sized), points: &[PolarPoint], closing: Option<PolarPoint>, interpolation: Interpolation, rings: RingShape) {
    let mut all: Vec<PolarPoint> = points.to_vec();
    all.extend(closing);

    match interpolation {
        Interpolation::Linear | Interpolation::Smooth { tension: 0.0 } => {
            for point in &all[1..] {
                path.line_to(point.position);
            }
        }
        Interpolation::Smooth { .. } | Interpolation::Cardinal { .. } | Interpolation::CatmullRom { .. } => {
            trace_spline(path, &all, closing.is_some(), interpolation)
        }
        Interpolation::Monotone => trace_spline(path, &all, closing.is_some(), Interpolation::Smooth { tension: LOOP_TENSION }),
        Interpolation::Stepped(stepped) => {
            for pair in all.windows(2) {
                trace_step(path, pair[0], pair[1], stepped, rings);
            }
        }
    }
}

/// The spline `interpolation` draws through `points`, from the first, where the path is, with
/// each point's control points from its neighbours, as for cartesian lines (see
/// `spline_controls_at`). When
/// `closed`, the last point is the first again, and the neighbours wrap around, so the loop is
/// smooth where it closes too.
fn trace_spline(path: &mut (impl PathSink + ?Sized), points: &[PolarPoint], closed: bool, interpolation: Interpolation) {
    let count = points.len();
    if count < 2 {
        return;
    }

    // Around a loop the last point is the first; its neighbours are the second and the one
    // before the last.
    let neighbours = |index: usize| -> (Position, Position) {
        let previous = if index > 0 {
            points[index - 1].position
        } else if closed {
            points[count - 2].position
        } else {
            points[0].position
        };

        let next = if index + 1 < count {
            points[index + 1].position
        } else if closed {
            points[1].position
        } else {
            points[count - 1].position
        };

        (previous, next)
    };

    let controls: Vec<(Position, Position)> = (0..count)
        .map(|index| {
            let (previous, next) = neighbours(index);
            spline_controls_at(interpolation, previous, points[index].position, next)
        })
        .collect();

    for index in 1..count {
        path.bezier_curve_to(controls[index - 1].1, controls[index].0, points[index].position);
    }
}

/// One step from `from`, where the path is, to `to`, around the rings of `rings` and along the
/// spokes: `After` goes around the ring at `from`'s distance, then along `to`'s spoke; `Before`
/// along `from`'s spoke, then around the ring at `to`'s distance; `Middle` turns halfway round,
/// and `MiddleVertical` halfway out. Around circles the turns are arcs, around polygons straight
/// lines from corner to corner, so steps follow the grid.
fn trace_step(path: &mut (impl PathSink + ?Sized), from: PolarPoint, to: PolarPoint, stepped: Stepped, rings: RingShape) {
    match stepped {
        Stepped::After => {
            rings.at(from.distance).trace(path, from.angle, to.angle);
            path.line_to(to.position);
        }
        Stepped::Before => {
            let ring = rings.at(to.distance);
            path.line_to(ring.point(from.angle));
            ring.trace(path, from.angle, to.angle);
        }
        Stepped::Middle => {
            let middle = (from.angle + to.angle) / 2.0;
            let (inner, outer) = (rings.at(from.distance), rings.at(to.distance));
            inner.trace(path, from.angle, middle);
            path.line_to(outer.point(middle));
            outer.trace(path, middle, to.angle);
        }
        Stepped::MiddleVertical => {
            let ring = rings.at((from.distance + to.distance) / 2.0);
            path.line_to(ring.point(from.angle));
            ring.trace(path, from.angle, to.angle);
            path.line_to(to.position);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI, TAU};
    use carbide::draw::{Position, Scalar};
    use crate::controller::line_path::PathSink;
    use crate::element::{Interpolation, Stepped};
    use crate::scale::{Axis, Circle, LinearScale, Polar, PolarRole};
    use crate::Scale;
    use crate::scale::polar::RingShape;
    use super::{trace_polar_line, trace_polar_line_reversed, PolarPoint};

    /// A full circle of radius 100 around the origin, without a hole.
    fn polar() -> Polar {
        let mut angle = LinearScale::new(Axis::Radial);
        angle.set_polar_role(Some(PolarRole::Angle));
        let mut radius = LinearScale::new(Axis::Radial);
        radius.set_polar_role(Some(PolarRole::Radius));

        Polar::new(&angle, &radius, Circle { center: Position::new(0.0, 0.0), radius: 100.0 })
    }

    /// Circle rings around `polar`.
    fn circles(polar: &Polar) -> RingShape<'_> {
        RingShape { polar, corners: None }
    }

    fn point(angle: Scalar, distance: Scalar) -> PolarPoint {
        let (sin, cos) = angle.sin_cos();
        PolarPoint { position: Position::new(distance * sin, -distance * cos), angle, distance }
    }

    fn close(a: Position, b: Position) -> bool {
        (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6
    }

    /// A straight loop is chords from spoke to spoke, back to where it started.
    #[test]
    fn straight_loops_close_with_a_chord() {
        let points = [point(0.0, 10.0), point(FRAC_PI_2, 20.0), point(PI, 10.0)];
        let closing = PolarPoint { angle: TAU, ..points[0] };

        let polar = polar();
        let mut traced = Traced::default();
        trace_polar_line(&mut traced, &points, Some(closing), Interpolation::Linear, circles(&polar));

        assert_eq!(traced.commands.len(), 4, "move, two chords, and the chord back");
        assert!(close(traced.commands[3].1, points[0].position));
    }

    /// A smooth loop has no ends: where it closes, the curve leaves the first point along the
    /// same tangent it arrives on, so there is no corner.
    #[test]
    fn smooth_loops_have_no_corner_where_they_close() {
        let points = [point(0.0, 10.0), point(TAU / 3.0, 14.0), point(2.0 * TAU / 3.0, 8.0)];
        let closing = PolarPoint { angle: TAU, ..points[0] };

        let polar = polar();
        for interpolation in [Interpolation::Smooth { tension: 0.4 }, Interpolation::Cardinal { tension: 0.2 }, Interpolation::CatmullRom { alpha: 0.5 }] {
            let mut traced = Traced::default();
            trace_polar_line(&mut traced, &points, Some(closing), interpolation, circles(&polar));

            let curves: Vec<&[Position]> = traced.curves.iter().map(|curve| curve.as_slice()).collect();
            assert_eq!(curves.len(), 3);

            // The arriving control point, the first point, and the leaving one are in a line.
            let arriving = curves[2][1];
            let leaving = curves[0][0];
            let first = points[0].position;
            let cross = (first.x - arriving.x) * (leaving.y - first.y) - (first.y - arriving.y) * (leaving.x - first.x);
            assert!(cross.abs() < 1e-6, "{interpolation:?}: the tangents line up: {}", cross);
        }
    }

    /// `Stepped::After` runs along the arc at the distance it leaves from, then along the next
    /// point's spoke.
    #[test]
    fn steps_follow_the_circle_then_the_spoke() {
        let points = [point(0.0, 10.0), point(FRAC_PI_2, 20.0)];

        let polar = polar();
        let mut traced = Traced::default();
        trace_polar_line(&mut traced, &points, None, Interpolation::Stepped(Stepped::After), circles(&polar));

        let arc_end = traced.curves[0][2];
        assert!(close(arc_end, Position::new(10.0, 0.0)), "a quarter turn at distance 10 ends on the right: {:?}", arc_end);
        assert!(close(traced.commands.last().unwrap().1, points[1].position), "then out along the spoke");
    }

    /// Traced backwards, a step turns the other way round, so it follows the same path.
    #[test]
    fn reversed_steps_follow_the_same_path() {
        let points = [point(0.0, 10.0), point(FRAC_PI_2, 20.0)];

        let polar = polar();
        let mut traced = Traced::default();
        trace_polar_line_reversed(&mut traced, &points, Interpolation::Stepped(Stepped::After), circles(&polar));

        // Back along the spoke to distance 10 first, then the arc back to the top.
        assert!(close(traced.commands[1].1, Position::new(10.0, 0.0)), "{:?}", traced.commands);
        assert!(close(traced.curves[0][2], points[0].position));
    }

    /// Around polygon rings, steps run straight from corner to corner instead of along arcs:
    /// `After` goes straight to the next corner at the distance it leaves from, then out along
    /// the spoke, and `Middle` turns on the polygon's edge halfway between the corners.
    #[test]
    fn steps_around_polygons_are_straight() {
        let polar = polar();
        let corners = [0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2];
        let polygons = RingShape { polar: &polar, corners: Some(&corners) };
        let points = [point(0.0, 10.0), point(FRAC_PI_2, 20.0)];

        let mut after = Traced::default();
        trace_polar_line(&mut after, &points, None, Interpolation::Stepped(Stepped::After), polygons);
        assert!(after.curves.is_empty(), "no arcs");
        assert_eq!(after.commands.len(), 3, "move, along the edge, out along the spoke: {:?}", after.commands);
        assert!(close(after.commands[1].1, Position::new(10.0, 0.0)), "the next corner at distance 10");
        assert!(close(after.commands[2].1, points[1].position));

        let mut middle = Traced::default();
        trace_polar_line(&mut middle, &points, None, Interpolation::Stepped(Stepped::Middle), polygons);
        assert!(middle.curves.is_empty(), "no arcs");
        assert!(close(middle.commands[1].1, Position::new(5.0, -5.0)), "halfway along the edge at 10: {:?}", middle.commands);
        assert!(close(middle.commands[2].1, Position::new(10.0, -10.0)), "out to the edge at 20");
        assert!(close(middle.commands[3].1, points[1].position), "then along it to the corner");
    }

    #[derive(Default)]
    struct Traced {
        /// Each command's kind and end point, in order.
        commands: Vec<(&'static str, Position)>,
        /// Each curve's two control points and end point.
        curves: Vec<Vec<Position>>,
    }

    impl PathSink for Traced {
        fn move_to(&mut self, to: Position) {
            self.commands.push(("move", to));
        }

        fn line_to(&mut self, to: Position) {
            self.commands.push(("line", to));
        }

        fn bezier_curve_to(&mut self, control1: Position, control2: Position, to: Position) {
            self.commands.push(("curve", to));
            self.curves.push(vec![control1, control2, to]);
        }
    }
}
