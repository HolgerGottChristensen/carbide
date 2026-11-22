//! The shapes data is drawn as, and how a position is tested against them.

use std::f64::consts::{PI, TAU};
use std::ops::Range;
use std::rc::Rc;
use carbide::draw::{Position, Scalar};
use carbide_core::widget::canvas::CanvasContext;
use crate::controller::line_path::PathSink;
use crate::controller::orientation::{Oriented, Orientation};
use crate::scale::Polar;
use crate::scale::polar::RingShape;

/// How far from a marker or a line, in pixels, a position still hits it.
pub(crate) const LINE_TOLERANCE: Scalar = 4.0;

/// A shape a datum is drawn as, in the chart's own pixels.
#[derive(Clone, Debug)]
pub(crate) enum Shape {
    /// A filled circle around `center`, such as a line's point or a bubble.
    Marker { center: Position, radius: Scalar },
    /// A rectangle in the index frame of `orientation` (see `Orientation`): from `side` to
    /// `side + thickness` along the index axis, and from `from` to `to` along the value axis.
    Rect { orientation: Orientation, side: Scalar, thickness: Scalar, from: Scalar, to: Scalar },
    /// The part of a ring around a polar chart's centre between two screen angles and two
    /// distances, in the order they were given: its inner and outer edges are circles, or polygons
    /// with a corner on each of `corners` (see `RingShape`).
    Wedge { polar: Polar, corners: Option<Rc<[Scalar]>>, angles: (Scalar, Scalar), distances: (Scalar, Scalar) },
    /// A dataset's line through its points, and the areas it fills.
    Polyline(Polyline),
}

/// A line through a dataset's points, as straight pieces between them, with the data each point
/// shows (see `Datum`).
#[derive(Clone, Debug)]
pub(crate) struct Polyline {
    pub(crate) vertices: Vec<(Position, super::Datum)>,
    /// The unbroken runs of the line, as ranges of `vertices`.
    pub(crate) runs: Vec<Range<usize>>,
    /// Whether the line closes into a loop, from its last point back to its first.
    pub(crate) closed: bool,
    /// The areas the line fills, as polygons: a position inside an odd number of them is inside
    /// the area, so a ring is two loops.
    pub(crate) areas: Vec<Vec<Position>>,
}

/// Where a position hits a shape: the datum it hits, by its index among a polyline's vertices
/// (0 for shapes of one datum), and how far from the shape it is, 0 inside.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Hit {
    pub(crate) vertex: usize,
    pub(crate) distance: Scalar,
}

impl Shape {
    /// Adds the shape's outline to the current path, closed. A polyline adds nothing, as lines
    /// are drawn with their interpolation by their controllers.
    pub(crate) fn trace(&self, ctx: &mut CanvasContext) {
        match self {
            Shape::Marker { center, radius } => {
                // The canvas takes a circle's diameter.
                ctx.circle(center.x, center.y, radius * 2.0);
                return;
            }
            Shape::Rect { orientation, side, thickness, from, to } => {
                let mut path = Oriented { path: &mut *ctx, orientation: *orientation };
                path.move_to(Position::new(*side, *from));
                path.line_to(Position::new(side + thickness, *from));
                path.line_to(Position::new(side + thickness, *to));
                path.line_to(Position::new(*side, *to));
            }
            Shape::Wedge { polar, corners, angles: (a1, a2), distances: (d1, d2) } => {
                let rings = RingShape { polar, corners: corners.as_deref() };
                let (outer, inner) = (rings.at(d1.max(*d2)), rings.at(d1.min(*d2)));
                PathSink::move_to(ctx, outer.point(*a1));
                outer.trace(ctx, *a1, *a2);
                PathSink::line_to(ctx, inner.point(*a2));
                inner.trace(ctx, *a2, *a1);
            }
            Shape::Polyline(_) => return,
        }
        ctx.close_path();
    }

    /// The point a datum drawn as this shape is measured from, for the nearest datum: a marker's
    /// centre, a rectangle's middle, or the middle of a wedge, halfway round and halfway out. A
    /// polyline's data are measured from its vertices instead.
    pub(crate) fn center(&self) -> Option<Position> {
        match self {
            Shape::Marker { center, .. } => Some(*center),
            Shape::Rect { orientation, side, thickness, from, to } => Some(orientation.swap(Position::new(side + thickness / 2.0, (from + to) / 2.0))),
            Shape::Wedge { polar, corners, angles: (a1, a2), distances: (d1, d2) } => {
                let angle = (a1 + a2) / 2.0;
                let rings = RingShape { polar, corners: corners.as_deref() };
                let reach = rings.at(d1.max(*d2)).reach(angle);
                Some(polar.at(angle, (d1 + d2) / 2.0 * reach))
            }
            Shape::Polyline(_) => None,
        }
    }

    /// The box around the shape on the screen, as `(left, top, right, bottom)`, the top the smaller
    /// y: a marker's square, a rectangle's corners, and a wedge's outer ring sampled round its
    /// angles. `None` for a polyline, whose points are measured one by one.
    pub(crate) fn bounds(&self) -> Option<(Scalar, Scalar, Scalar, Scalar)> {
        let around = |points: &mut dyn Iterator<Item=Position>| {
            points.fold(None, |bounds: Option<(Scalar, Scalar, Scalar, Scalar)>, point| {
                Some(match bounds {
                    None => (point.x, point.y, point.x, point.y),
                    Some((left, top, right, bottom)) => (left.min(point.x), top.min(point.y), right.max(point.x), bottom.max(point.y)),
                })
            })
        };

        match self {
            Shape::Marker { center, radius } => Some((center.x - radius, center.y - radius, center.x + radius, center.y + radius)),
            Shape::Rect { orientation, side, thickness, from, to } => around(&mut [(*side, *from), (side + thickness, *to)].into_iter().map(|(x, y)| orientation.swap(Position::new(x, y)))),
            Shape::Wedge { polar, angles: (a1, a2), distances: (d1, d2), .. } => {
                const STEPS: usize = 8;
                let outer = d1.max(*d2);
                let mut points = (0..=STEPS).map(|step| polar.at(a1 + (a2 - a1) * step as Scalar / STEPS as Scalar, outer)).chain(std::iter::once(polar.at(*a1, d1.min(*d2))));
                around(&mut points)
            }
            Shape::Polyline(_) => None,
        }
    }

    /// Where `position` hits the shape, or `None` when it misses: inside a rectangle, a wedge or
    /// a filled area, or within `LINE_TOLERANCE` of a marker or a line. A line hit between two
    /// points hits the nearer of the two; an area hit hits the point of its line nearest along
    /// the index axis (see `Along::gap`).
    pub(crate) fn hit(&self, position: Position) -> Option<Hit> {
        match self {
            Shape::Marker { center, radius } => {
                let distance = (distance(position, *center) - radius).max(0.0);
                (distance <= LINE_TOLERANCE).then_some(Hit { vertex: 0, distance })
            }
            Shape::Rect { orientation, side, thickness, from, to } => {
                let frame = orientation.swap(position);
                let (left, right) = (side.min(side + thickness), side.max(side + thickness));
                let (low, high) = (from.min(*to), from.max(*to));
                (left <= frame.x && frame.x <= right && low <= frame.y && frame.y <= high).then_some(Hit { vertex: 0, distance: 0.0 })
            }
            Shape::Wedge { polar, corners, angles: (a1, a2), distances: (d1, d2) } => {
                let (angle, from_center) = polar.locate(position);
                let (low, high) = (a1.min(*a2), a1.max(*a2));

                // The angle the number of whole turns round that lands it on or after the wedge's
                // start; a wedge going all the way round holds every angle.
                let into = (angle - low).rem_euclid(TAU);
                if high - low < TAU - 1e-9 && into > high - low {
                    return None;
                }

                let rings = RingShape { polar, corners: corners.as_deref() };
                let reach = rings.at(d1.max(*d2)).reach(low + into);
                let (inner, outer) = (d1.min(*d2) * reach, d1.max(*d2) * reach);
                (inner <= from_center && from_center <= outer).then_some(Hit { vertex: 0, distance: 0.0 })
            }
            Shape::Polyline(line) => line.hit(position),
        }
    }
}

impl Polyline {
    /// Where `position` hits the line or its areas (see `Shape::hit`): the line first, as it is
    /// drawn over its area.
    fn hit(&self, position: Position) -> Option<Hit> {
        let mut nearest: Option<Hit> = None;

        for run in &self.runs {
            let mut segments: Vec<(usize, usize)> = (run.start..run.end.saturating_sub(1)).map(|i| (i, i + 1)).collect();
            if self.closed && run.len() > 2 {
                segments.push((run.end - 1, run.start));
            }
            // A run of one point is a point.
            if run.len() == 1 {
                segments.push((run.start, run.start));
            }

            for (a, b) in segments {
                let (from, to) = (self.vertices[a].0, self.vertices[b].0);
                let distance = segment_distance(position, from, to);
                if distance > LINE_TOLERANCE || nearest.is_some_and(|hit| hit.distance <= distance) {
                    continue;
                }

                let vertex = if self::distance(position, from) <= self::distance(position, to) { a } else { b };
                nearest = Some(Hit { vertex, distance });
            }
        }

        if nearest.is_some() {
            return nearest;
        }

        let inside = self.areas.iter().filter(|polygon| contains(polygon, position)).count() % 2 == 1;
        if !inside {
            return None;
        }

        // Inside the area: the point of the line nearest along the index axis, as the area hangs
        // from it.
        self.vertices.iter().enumerate()
            .map(|(vertex, (point, datum))| {
                let gap = datum.place.as_ref().map_or_else(|| self::distance(position, *point), |place| place.along.gap(position));
                (vertex, gap)
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(vertex, _)| Hit { vertex, distance: 0.0 })
    }
}

/// Where a datum sits along the index axis, on the screen, for picking the data at one category
/// (see `PickMode::Index`): an x or a y along a side, or, around a circle, a screen angle (see
/// `Polar`) or a distance from the centre.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Along {
    X(Scalar),
    Y(Scalar),
    Angle { center: Position, angle: Scalar },
    Distance { center: Position, distance: Scalar },
}

impl Along {
    /// How far `position` is from the datum along the index axis, in pixels. Around a circle, the
    /// angle between them is measured along the circle through `position`.
    pub(crate) fn gap(&self, position: Position) -> Scalar {
        match *self {
            Along::X(x) => (position.x - x).abs(),
            Along::Y(y) => (position.y - y).abs(),
            Along::Angle { center, angle } => {
                let (dx, dy) = (position.x - center.x, position.y - center.y);
                let between = ((dx.atan2(-dy) - angle + PI).rem_euclid(TAU) - PI).abs();
                between * dx.hypot(dy)
            }
            Along::Distance { center, distance } => (self::distance(position, center) - distance).abs(),
        }
    }

    /// The screen coordinate along a side, with the side: `Some((true, x))` along x and
    /// `Some((false, y))` along y; `None` around a circle.
    pub(crate) fn coordinate(&self) -> Option<(bool, Scalar)> {
        match *self {
            Along::X(x) => Some((true, x)),
            Along::Y(y) => Some((false, y)),
            Along::Angle { .. } | Along::Distance { .. } => None,
        }
    }
}

pub(crate) fn distance(a: Position, b: Position) -> Scalar {
    (a.x - b.x).hypot(a.y - b.y)
}

/// How far `position` is from the straight piece from `from` to `to`.
fn segment_distance(position: Position, from: Position, to: Position) -> Scalar {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        return distance(position, from);
    }

    let t = (((position.x - from.x) * dx + (position.y - from.y) * dy) / length_squared).clamp(0.0, 1.0);
    distance(position, Position::new(from.x + t * dx, from.y + t * dy))
}

/// Whether `position` is inside `polygon`, by the even-odd rule.
fn contains(polygon: &[Position], position: Position) -> bool {
    let mut inside = false;
    let mut previous = match polygon.last() {
        Some(last) => *last,
        None => return false,
    };

    for &point in polygon {
        if (point.y > position.y) != (previous.y > position.y) {
            let x = point.x + (position.y - point.y) * (previous.x - point.x) / (previous.y - point.y);
            if position.x < x {
                inside = !inside;
            }
        }
        previous = point;
    }

    inside
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};
    use std::rc::Rc;
    use carbide::draw::{Position, Scalar};
    use crate::controller::orientation::Orientation;
    use crate::geometry::Datum;
    use crate::scale::{Axis, CategoryScale, Circle, LinearScale, Polar, PolarRole, Scale};
    use super::{Along, Hit, Polyline, Shape, LINE_TOLERANCE};

    /// A marker is hit on it and within the tolerance around its edge, measured from the edge.
    #[test]
    fn markers_are_hit_near_their_edge() {
        let marker = Shape::Marker { center: Position::new(100.0, 100.0), radius: 5.0 };

        assert_eq!(marker.hit(Position::new(100.0, 100.0)), Some(Hit { vertex: 0, distance: 0.0 }));
        assert_eq!(marker.hit(Position::new(108.0, 100.0)), Some(Hit { vertex: 0, distance: 3.0 }));
        assert_eq!(marker.hit(Position::new(100.0, 100.0 + 5.0 + LINE_TOLERANCE + 0.5)), None);
    }

    /// A rectangle is hit inside only, in its index frame: a lying bar's from and to run along x
    /// on the screen.
    #[test]
    fn rectangles_are_hit_inside() {
        let standing = Shape::Rect { orientation: Orientation::Vertical, side: 10.0, thickness: 20.0, from: 200.0, to: 50.0 };
        assert!(standing.hit(Position::new(15.0, 100.0)).is_some());
        assert!(standing.hit(Position::new(35.0, 100.0)).is_none(), "past its far side");
        assert!(standing.hit(Position::new(15.0, 40.0)).is_none(), "above its head");
        assert_eq!(standing.center(), Some(Position::new(20.0, 125.0)));

        let lying = Shape::Rect { orientation: Orientation::Horizontal, side: 10.0, thickness: 20.0, from: 0.0, to: 150.0 };
        assert!(lying.hit(Position::new(100.0, 15.0)).is_some());
        assert!(lying.hit(Position::new(15.0, 100.0)).is_none());
        assert_eq!(lying.center(), Some(Position::new(75.0, 20.0)));
    }

    fn polar() -> Polar {
        let mut angle = CategoryScale::new(Axis::Radial).labels(["A", "B", "C", "D"]);
        angle.set_polar_role(Some(PolarRole::Angle));
        angle.set_range(0.0, 3.0);
        let radius = LinearScale::new(Axis::Radial);
        Polar::new(&angle, &radius, Circle { center: Position::new(200.0, 200.0), radius: 100.0 })
    }

    /// A wedge is hit between its angles and its distances, also when its angles run past a full
    /// turn or backwards, and one all the way round holds every angle.
    #[test]
    fn wedges_are_hit_between_their_angles_and_distances() {
        let polar = polar();
        // The quarter from the top to the right, from 50 to 100 pixels out.
        let wedge = Shape::Wedge { polar, corners: None, angles: (0.0, FRAC_PI_2), distances: (50.0, 100.0) };

        assert!(wedge.hit(Position::new(250.0, 150.0)).is_some(), "up and to the right, 70 out");
        assert!(wedge.hit(Position::new(150.0, 150.0)).is_none(), "up and to the left");
        assert!(wedge.hit(Position::new(220.0, 180.0)).is_none(), "in the hole, 28 out");
        assert!(wedge.hit(Position::new(290.0, 110.0)).is_none(), "outside, 127 out");

        let backwards = Shape::Wedge { polar, corners: None, angles: (2.0 * PI + FRAC_PI_2, 2.0 * PI), distances: (100.0, 50.0) };
        assert!(backwards.hit(Position::new(250.0, 150.0)).is_some(), "the same quarter, a turn on and backwards");

        let around = Shape::Wedge { polar, corners: None, angles: (0.0, 2.0 * PI), distances: (0.0, 100.0) };
        assert!(around.hit(Position::new(150.0, 250.0)).is_some());
    }

    /// A polygon ring's wedge is hit inside its straight edges, which cut inside the circle
    /// between the corners.
    #[test]
    fn polygon_wedges_follow_their_edges() {
        let polar = polar();
        let corners: Rc<[Scalar]> = Rc::from(vec![0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2]);
        let wedge = Shape::Wedge { polar, corners: Some(corners), angles: (0.0, FRAC_PI_2), distances: (0.0, 100.0) };

        // Halfway between the corners at the top and the right, the edge is 71 out.
        let between = |out: Scalar| Position::new(200.0 + out / 2.0_f64.sqrt(), 200.0 - out / 2.0_f64.sqrt());
        assert!(wedge.hit(between(65.0)).is_some());
        assert!(wedge.hit(between(80.0)).is_none(), "past the edge, though inside the circle");
    }

    fn datum(index: usize, x: Scalar) -> Datum {
        Datum { index, place: Some(super::super::IndexPlace { scale: 0, value: index as Scalar, along: Along::X(x) }) }
    }

    /// A line is hit within the tolerance of its pieces, at the nearer of the two points; its
    /// area is hit inside, at the point nearest along the index axis.
    #[test]
    fn lines_hit_their_nearer_point_and_areas_the_point_above() {
        let line = Polyline {
            vertices: vec![
                (Position::new(0.0, 100.0), datum(0, 0.0)),
                (Position::new(100.0, 0.0), datum(1, 100.0)),
                (Position::new(200.0, 100.0), datum(2, 200.0)),
            ],
            runs: vec![0..3],
            closed: false,
            areas: vec![vec![Position::new(0.0, 100.0), Position::new(100.0, 0.0), Position::new(200.0, 100.0), Position::new(200.0, 200.0), Position::new(0.0, 200.0)]],
        };
        let shape = Shape::Polyline(line);

        assert_eq!(shape.hit(Position::new(70.0, 31.0)).map(|hit| hit.vertex), Some(1), "on the first piece, nearer the second point");
        assert_eq!(shape.hit(Position::new(20.0, 81.0)).map(|hit| hit.vertex), Some(0));
        assert_eq!(shape.hit(Position::new(160.0, 150.0)).map(|hit| hit.vertex), Some(2), "in the area, under the third point");
        assert_eq!(shape.hit(Position::new(30.0, 20.0)), None, "above the line");
    }

    /// Along a side the gap is the distance along it; around a circle, the arc between the two
    /// angles at the position's distance, or the difference in distance.
    #[test]
    fn gaps_along_the_index_axis() {
        assert_eq!(Along::X(10.0).gap(Position::new(15.0, 99.0)), 5.0);
        assert_eq!(Along::Y(10.0).gap(Position::new(99.0, 4.0)), 6.0);

        let center = Position::new(0.0, 0.0);
        let gap = Along::Angle { center, angle: 0.0 }.gap(Position::new(100.0, 0.0));
        assert!((gap - 100.0 * FRAC_PI_2).abs() < 1e-9, "a quarter turn, 100 out: {gap}");
        let gap = Along::Angle { center, angle: 0.1 }.gap(Position::new(-1.0, -100.0));
        assert!(gap < 15.0, "across the top, not the long way round: {gap}");

        assert_eq!(Along::Distance { center, distance: 30.0 }.gap(Position::new(0.0, 50.0)), 20.0);
    }
}
