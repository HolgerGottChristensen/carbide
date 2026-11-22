//! The geometry of a polar chart: where an angle and a radius fall on the screen, how the circle
//! is fitted inside the chart area, and arcs traced as curves.
//!
//! Angles here are on the screen, in radians, measured clockwise from the top of the circle, so
//! a direction `(sin a, -cos a)` points from the centre towards angle `a` (the screen's y grows
//! downwards).

use std::f64::consts::{FRAC_PI_2, TAU};
use carbide::draw::{Dimension, Position, Rect, Scalar};
use crate::controller::line_path::PathSink;
use crate::scale::{AngleDirection, Scale};

/// The gap between the rim and the labels around it.
pub(crate) const RIM_LABEL_GAP: Scalar = 6.0;

/// The circle a chart's polar scales share, fitted inside the chart area on every render.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Circle {
    pub center: Position,
    pub radius: Scalar,
}

/// Maps a dataset's angle and radius onto the chart's circle, from an angle scale's start angle,
/// end angle and direction and a radius scale's inner radius (see `AxisOptions`). Controllers draw
/// polar datasets through it; `Scales::polar` gives the one for a pair of scales.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Polar {
    circle: Circle,
    /// The hole in the middle, in pixels.
    inner: Scalar,
    /// The screen angle of a normalized 0 along the angle scale.
    start: Scalar,
    /// How far the angle scale reaches from its start, on the screen: negative counter-clockwise.
    sweep: Scalar,
    full_circle: bool,
}

impl Polar {
    /// The mapping of `angle_scale` and `radius_scale` onto `circle`.
    pub(crate) fn new(angle_scale: &dyn Scale, radius_scale: &dyn Scale, circle: Circle) -> Polar {
        let (start, sweep) = angle_range(angle_scale);

        Polar {
            circle,
            inner: circle.radius * radius_scale.axis_options().inner_radius(),
            start,
            sweep,
            full_circle: angle_scale.axis_options().is_full_circle(),
        }
    }

    /// The circle the chart's polar scales share.
    pub fn circle(&self) -> Circle {
        self.circle
    }

    /// The radius of the hole in the middle, in pixels: 0 without one.
    pub fn inner_radius(&self) -> Scalar {
        self.inner
    }

    /// Whether the angle scale goes all the way around, so lines through it close into loops.
    pub fn is_full_circle(&self) -> bool {
        self.full_circle
    }

    /// The screen angle, in radians clockwise from the top, of a normalized position along the
    /// angle scale (see `Scale::normalize`).
    pub fn angle(&self, normalized: Scalar) -> Scalar {
        self.start + normalized * self.sweep
    }

    /// The start and end screen angles of the angle scale, in the order it runs.
    pub fn angle_range(&self) -> (Scalar, Scalar) {
        (self.start, self.start + self.sweep)
    }

    /// The distance from the centre, in pixels, of a normalized position along the radius scale:
    /// 0 at the inner radius and 1 at the rim.
    pub fn distance(&self, normalized: Scalar) -> Scalar {
        self.inner + normalized * (self.circle.radius - self.inner)
    }

    /// The point at screen angle `angle`, `distance` pixels from the centre.
    pub fn at(&self, angle: Scalar, distance: Scalar) -> Position {
        let (sin, cos) = angle.sin_cos();
        Position::new(self.circle.center.x + distance * sin, self.circle.center.y - distance * cos)
    }

    /// Where a point is drawn, from its normalized positions along the angle and radius scales.
    pub fn position(&self, angle: Scalar, radius: Scalar) -> Position {
        self.at(self.angle(angle), self.distance(radius))
    }

    /// The screen angle and the distance from the centre of `position`: the inverse of `at`. The
    /// angle is in radians clockwise from the top, from -π to π; the centre itself is at angle 0.
    pub fn locate(&self, position: Position) -> (Scalar, Scalar) {
        let (dx, dy) = (position.x - self.circle.center.x, position.y - self.circle.center.y);
        (dx.atan2(-dy), dx.hypot(dy))
    }

    /// The normalized position along the angle scale of the screen angle `angle`: the inverse of
    /// `angle`, taken the whole number of turns round that lands it in the sweep. An angle outside
    /// a sweep of less than a full turn is taken to the nearer end, so it is just below 0 or just
    /// above 1.
    pub fn normalized_angle(&self, angle: Scalar) -> Scalar {
        if self.sweep == 0.0 {
            return 0.0;
        }

        // A whole turn, in normalized units.
        let turn = TAU / self.sweep.abs();
        let normalized = ((angle - self.start) / self.sweep).rem_euclid(turn);

        if normalized > 1.0 && normalized - 1.0 > turn - normalized {
            normalized - turn
        } else {
            normalized
        }
    }

    /// The normalized position along the radius scale of `distance` pixels from the centre: the
    /// inverse of `distance`, 0 at the inner radius and 1 at the rim, below 0 in the hole and
    /// above 1 outside the circle.
    pub fn normalized_distance(&self, distance: Scalar) -> Scalar {
        let width = self.circle.radius - self.inner;
        if width <= 0.0 {
            return 0.0;
        }

        (distance - self.inner) / width
    }

    /// The normalized positions along the angle and radius scales of `position`: the inverse of
    /// `position`.
    pub fn normalized_at(&self, position: Position) -> (Scalar, Scalar) {
        let (angle, distance) = self.locate(position);
        (self.normalized_angle(angle), self.normalized_distance(distance))
    }
}

/// The screen angle `angle_scale` starts at, and how far it sweeps from there: negative when it
/// runs counter-clockwise. The start is clockwise from the top whatever the direction, so turning
/// the direction keeps the first category where it is; the end is as far from the start as it
/// says, the other way round.
pub(crate) fn angle_range(angle_scale: &dyn Scale) -> (Scalar, Scalar) {
    let options = angle_scale.axis_options();
    let sign = match options.direction() {
        AngleDirection::Clockwise => 1.0,
        AngleDirection::CounterClockwise => -1.0,
    };

    let start = options.start_angle().radians();
    let end = options.end_angle().radians();

    (start, sign * (end - start))
}

/// The shape of a polar chart's rings at any distance: circles, or polygons with a corner on
/// each of `corners`. Lines step around the circle along it, so steps follow the grid.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RingShape<'a> {
    pub(crate) polar: &'a Polar,
    pub(crate) corners: Option<&'a [Scalar]>,
}

impl<'a> RingShape<'a> {
    /// The ring `distance` from the centre.
    pub(crate) fn at(&self, distance: Scalar) -> Ring<'a> {
        Ring { polar: self.polar, distance, corners: self.corners }
    }
}

/// A ring `distance` from the centre of a polar chart: a circle, or, with `corners` (screen
/// angles in the order the angle scale runs), a polygon with a corner on each of those spokes.
/// Grid rings, the rim, and the rings areas fill to are all traced through it, so they keep one
/// shape.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Ring<'a> {
    pub(crate) polar: &'a Polar,
    pub(crate) distance: Scalar,
    pub(crate) corners: Option<&'a [Scalar]>,
}

impl Ring<'_> {
    /// The corners, each a whole number of turns either way on a full circle, so a stretch of the
    /// ring that goes past the start finds the corners beyond it.
    fn all_corners(&self) -> Vec<Scalar> {
        let Some(corners) = self.corners else {
            return vec![];
        };

        let turns: &[Scalar] = if self.polar.is_full_circle() { &[-TAU, 0.0, TAU] } else { &[0.0] };
        let mut all: Vec<Scalar> = turns.iter().flat_map(|turn| corners.iter().map(move |corner| corner + turn)).collect();
        all.sort_by(|a, b| a.total_cmp(b));
        all
    }

    /// Where the ring crosses the spoke at screen angle `angle`: on the circle, or on the polygon's
    /// edge between the corners on either side, or the circle outside the corners.
    pub(crate) fn point(&self, angle: Scalar) -> Position {
        let corners = self.all_corners();
        let around = corners.windows(2).find(|pair| pair[0] - 1e-9 <= angle && angle <= pair[1] + 1e-9);

        let distance = match around {
            Some(pair) => {
                let half = (pair[1] - pair[0]) / 2.0;
                let middle = pair[0] + half;
                self.distance * half.cos() / (angle - middle).cos()
            }
            None => self.distance,
        };

        self.polar.at(angle, distance)
    }

    /// How far the ring is from the centre on the spoke at screen angle `angle`, as a share of
    /// its distance: 1 on a circle and at a polygon's corners, less between them, where its edges
    /// cut inside the circle.
    pub(crate) fn reach(&self, angle: Scalar) -> Scalar {
        let at = self.point(angle);
        let center = self.polar.circle().center;
        let reach = ((at.x - center.x).powi(2) + (at.y - center.y).powi(2)).sqrt() / self.distance;
        if reach.is_finite() && reach > 0.0 { reach } else { 1.0 }
    }

    /// The unit direction the ring runs in at screen angle `angle`, heading the way `heading`'s
    /// sign says: towards greater angles when positive. On a polygon it is the edge on that side,
    /// so at a corner it is the edge the ring goes on to.
    pub(crate) fn tangent(&self, angle: Scalar, heading: Scalar) -> Position {
        let sign = if heading < 0.0 { -1.0 } else { 1.0 };
        let probe = angle + sign * 1e-6;
        let corners = self.all_corners();

        if let Some(pair) = corners.windows(2).find(|pair| pair[0] <= probe && probe <= pair[1]) {
            let (from, to) = (self.polar.at(pair[0], self.distance), self.polar.at(pair[1], self.distance));
            let (dx, dy) = ((to.x - from.x) * sign, (to.y - from.y) * sign);
            let length = (dx * dx + dy * dy).sqrt();
            if length > 0.0 {
                return Position::new(dx / length, dy / length);
            }
        }

        // Clockwise on the screen, the circle at screen angle `a` runs along `(cos a, sin a)`.
        Position::new(angle.cos() * sign, angle.sin() * sign)
    }

    /// Traces along the ring from screen angle `from`, where the path is, to `to`, either way
    /// round: an arc, or straight from corner to corner.
    pub(crate) fn trace(&self, path: &mut (impl PathSink + ?Sized), from: Scalar, to: Scalar) {
        if self.corners.is_none() {
            trace_arc(path, self.polar.circle().center, self.distance, from, to);
            return;
        }

        let (low, high) = (from.min(to), from.max(to));
        let mut between: Vec<Scalar> = self.all_corners().into_iter().filter(|corner| *corner > low + 1e-9 && *corner < high - 1e-9).collect();
        if to < from {
            between.reverse();
        }

        for corner in between {
            path.line_to(self.point(corner));
        }
        path.line_to(self.point(to));
    }

    /// Adds the whole ring to the path as a closed shape of its own, from the end of the angle
    /// scale back to its start when `backwards`, and from its start to its end otherwise.
    pub(crate) fn trace_whole(&self, path: &mut (impl PathSink + ?Sized), backwards: bool) {
        let (start, end) = self.polar.angle_range();
        let (from, to) = if backwards { (end, start) } else { (start, end) };

        path.move_to(self.point(from));
        self.trace(path, from, to);
    }
}

/// The unit direction from the centre towards screen angle `angle`.
pub(crate) fn direction(angle: Scalar) -> Position {
    let (sin, cos) = angle.sin_cos();
    Position::new(sin, -cos)
}

/// Traces an arc around `center`, `radius` pixels out, from screen angle `from` to `to` (either
/// way round), as cubic curves of at most a quarter turn each, from wherever the path is: callers
/// move or draw to the arc's start first.
pub(crate) fn trace_arc(path: &mut (impl PathSink + ?Sized), center: Position, radius: Scalar, from: Scalar, to: Scalar) {
    let at = |angle: Scalar| {
        let (sin, cos) = angle.sin_cos();
        Position::new(center.x + radius * sin, center.y - radius * cos)
    };

    let pieces = ((to - from).abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let step = (to - from) / pieces as Scalar;
    // How far along each end's tangent its control point sits, for a circular arc of `step`.
    let reach = 4.0 / 3.0 * (step / 4.0).tan() * radius;

    for piece in 0..pieces {
        let a = from + step * piece as Scalar;
        let b = a + step;
        let (start, end) = (at(a), at(b));
        // The tangent, clockwise on the screen, at screen angle `a` is `(cos a, sin a)`.
        let control1 = Position::new(start.x + reach * a.cos(), start.y + reach * a.sin());
        let control2 = Position::new(end.x - reach * b.cos(), end.y - reach * b.sin());
        path.bezier_curve_to(control1, control2, end);
    }
}

/// A label around the rim: its screen angle, its measured size, and how far out from the rim it
/// starts.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RimLabel {
    pub(crate) angle: Scalar,
    pub(crate) size: Dimension,
    pub(crate) gap: Scalar,
}

/// The circle for the chart's polar scales inside `area`. The arc from `start` sweeping `sweep`
/// (screen angles), with a hole of `inner` times the radius, is as large as fits and centred in
/// the area by its bounding box, so a half circle uses the whole width. Then it shrinks until
/// every label in `labels`, `RIM_LABEL_GAP` outside the rim at its angle, fits in the area too,
/// staying centred, but never below half the radius it had without labels.
pub(crate) fn fit_circle(area: Rect, start: Scalar, sweep: Scalar, inner: Scalar, labels: &[RimLabel]) -> Circle {
    let (min, max) = arc_bounds(start, sweep, inner);
    let width = (max.x - min.x).max(1e-9);
    let height = (max.y - min.y).max(1e-9);

    let largest = (area.width() / width).min(area.height() / height).max(0.0);
    let middle = Position::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0);
    let area_center = Position::new(area.left() + area.width() / 2.0, area.bottom() + area.height() / 2.0);

    // The centre is `area_center - radius * middle`, so a label's edges move linearly with the
    // radius: `fixed + radius * per_radius`. Each edge that must stay inside the area bounds the
    // radius.
    let mut radius = largest;

    for label in labels {
        let direction = direction(label.angle);
        let anchor = Position::new(area_center.x + label.gap * direction.x, area_center.y + label.gap * direction.y);
        let per_radius = Position::new(direction.x - middle.x, direction.y - middle.y);
        let (left, right) = label_extent(direction.x, label.size.width);
        let (top, bottom) = label_extent(direction.y, label.size.height);

        // (fixed part, per-radius part, limit, whether the edge must stay below the limit)
        let edges = [
            (anchor.x + left, per_radius.x, area.left(), false),
            (anchor.x + right, per_radius.x, area.right(), true),
            (anchor.y + top, per_radius.y, area.bottom(), false),
            (anchor.y + bottom, per_radius.y, area.top(), true),
        ];

        for (fixed, slope, limit, below) in edges {
            let grows_past = if below { slope > 1e-9 } else { slope < -1e-9 };
            if grows_past {
                radius = radius.min((limit - fixed) / slope);
            }
        }
    }

    let radius = radius.max(largest / 2.0);

    Circle {
        center: Position::new(area_center.x - radius * middle.x, area_center.y - radius * middle.y),
        radius,
    }
}

/// How far a label reaches from its anchor along one screen direction, before and after it, when
/// it hangs off the rim in direction `towards` (one coordinate of the unit direction): beyond the
/// anchor when it points that way, before it when it points the other, and centred on it in
/// between.
fn label_extent(towards: Scalar, size: Scalar) -> (Scalar, Scalar) {
    if towards > 0.2 {
        (0.0, size)
    } else if towards < -0.2 {
        (-size, 0.0)
    } else {
        (-size / 2.0, size / 2.0)
    }
}

/// The bounding box, in unit-circle coordinates on the screen, of the sector from `start`
/// sweeping `sweep`, between `inner` and 1 from the centre: its corners, and the rim's extremes
/// (top, right, bottom, left) that fall within the sweep.
fn arc_bounds(start: Scalar, sweep: Scalar, inner: Scalar) -> (Position, Position) {
    let (from, to) = if sweep >= 0.0 { (start, start + sweep) } else { (start + sweep, start) };

    let mut points = vec![direction(from), direction(to)];
    let inner_points = [direction(from), direction(to)].map(|point| Position::new(point.x * inner, point.y * inner));

    if inner > 0.0 {
        points.extend(inner_points);
    } else {
        points.push(Position::new(0.0, 0.0));
    }

    // The quarter turns between `from` and `to`, where the rim reaches furthest out.
    let mut quarter = (from / FRAC_PI_2).ceil() * FRAC_PI_2;
    while quarter <= to + 1e-9 && quarter < from + TAU {
        points.push(direction(quarter));
        quarter += FRAC_PI_2;
    }

    let min = points.iter().fold(Position::new(Scalar::INFINITY, Scalar::INFINITY), |min, point| Position::new(min.x.min(point.x), min.y.min(point.y)));
    let max = points.iter().fold(Position::new(Scalar::NEG_INFINITY, Scalar::NEG_INFINITY), |max, point| Position::new(max.x.max(point.x), max.y.max(point.y)));

    (min, max)
}

/// Whether two screen angles point the same way, a whole number of turns apart.
pub(crate) fn same_direction(a: Scalar, b: Scalar) -> bool {
    let turns = (a - b) / TAU;
    (turns - turns.round()).abs() < 1e-9
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};
    use carbide::draw::{Angle, Dimension, Position, Rect, Scalar};
    use crate::controller::line_path::PathSink;
    use crate::scale::{AngleDirection, Axis, CategoryScale, LinearScale, PolarRole, Scale};
    use super::{fit_circle, trace_arc, Circle, Polar, RimLabel, Ring};

    fn close(a: Position, b: Position) -> bool {
        (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6
    }

    /// `scale` as the angle of its polar chart.
    fn angle(mut scale: LinearScale) -> LinearScale {
        scale.set_polar_role(Some(PolarRole::Angle));
        scale
    }

    fn unit_circle() -> Circle {
        Circle { center: Position::new(100.0, 100.0), radius: 50.0 }
    }

    fn radius_scale(inner: Scalar) -> LinearScale {
        let mut scale = LinearScale::new(Axis::Radial).inner_radius(inner);
        scale.set_polar_role(Some(PolarRole::Radius));
        scale.set_range(0.0, 10.0);
        scale
    }

    /// An angle scale starts at the top and runs clockwise by default: a quarter of the way
    /// round is the right of the circle, on the screen, whose y grows downwards.
    #[test]
    fn angles_start_at_the_top_and_run_clockwise() {
        let angle = angle(LinearScale::new(Axis::Radial));
        let polar = Polar::new(&angle, &radius_scale(0.0), unit_circle());

        assert!(close(polar.position(0.0, 1.0), Position::new(100.0, 50.0)), "the top");
        assert!(close(polar.position(0.25, 1.0), Position::new(150.0, 100.0)), "the right");
        assert!(close(polar.position(0.5, 0.5), Position::new(100.0, 125.0)), "halfway down");
        assert!(polar.is_full_circle());
    }

    /// Counter-clockwise, a quarter of the way round is the left. A start angle turns the whole
    /// scale, measured clockwise from the top whichever way it runs, so the first category stays
    /// where it was put when the direction changes.
    #[test]
    fn direction_and_start_angle_turn_the_scale() {
        let counter = angle(LinearScale::new(Axis::Radial).direction(AngleDirection::CounterClockwise));
        let polar = Polar::new(&counter, &radius_scale(0.0), unit_circle());
        assert!(close(polar.position(0.25, 1.0), Position::new(50.0, 100.0)));

        let turned = angle(LinearScale::new(Axis::Radial).start_angle(Angle::Degrees(90.0)));
        let polar = Polar::new(&turned, &radius_scale(0.0), unit_circle());
        assert!(close(polar.position(0.0, 1.0), Position::new(150.0, 100.0)), "starts on the right");
        assert!(!polar.is_full_circle(), "90° to 360° is three quarters of a circle");

        let both = angle(LinearScale::new(Axis::Radial).start_angle(Angle::Degrees(90.0)).end_angle(Angle::Degrees(450.0)).direction(AngleDirection::CounterClockwise));
        let polar = Polar::new(&both, &radius_scale(0.0), unit_circle());
        assert!(close(polar.position(0.0, 1.0), Position::new(150.0, 100.0)), "still starts on the right");
        assert!(close(polar.position(0.25, 1.0), Position::new(100.0, 50.0)), "and runs up to the top");
    }

    /// A partial sweep isn't a full circle, and maps 0..1 onto it.
    #[test]
    fn partial_sweeps_map_onto_their_arc() {
        let half = angle(LinearScale::new(Axis::Radial).start_angle(Angle::Degrees(-90.0)).end_angle(Angle::Degrees(90.0)));
        let polar = Polar::new(&half, &radius_scale(0.0), unit_circle());

        assert!(!polar.is_full_circle());
        assert!(close(polar.position(0.0, 1.0), Position::new(50.0, 100.0)), "starts on the left");
        assert!(close(polar.position(1.0, 1.0), Position::new(150.0, 100.0)), "ends on the right");
    }

    /// With a hole, a normalized 0 along the radius is the inner radius, not the centre.
    #[test]
    fn the_radius_starts_at_the_inner_radius() {
        let polar = Polar::new(&angle(LinearScale::new(Axis::Radial)), &radius_scale(0.4), unit_circle());

        assert_eq!(polar.inner_radius(), 20.0);
        assert_eq!(polar.distance(0.0), 20.0);
        assert_eq!(polar.distance(1.0), 50.0);
    }

    /// Around a full circle, categories are a sweep over their number apart, so the last sits
    /// one step before the first instead of on it; on a partial sweep the first and last sit on
    /// its ends.
    #[test]
    fn categories_around_a_full_circle_do_not_overlap() {
        let mut full = CategoryScale::new(Axis::Radial).labels(["a", "b", "c", "d"]);
        full.set_polar_role(Some(PolarRole::Angle));
        full.set_range(0.0, 3.0);
        assert_eq!([0.0, 1.0, 2.0, 3.0].map(|value| full.normalize(value)), [0.0, 0.25, 0.5, 0.75]);

        let mut half = CategoryScale::new(Axis::Radial).labels(["a", "b", "c"]).end_angle(Angle::Degrees(180.0));
        half.set_polar_role(Some(PolarRole::Angle));
        half.set_range(0.0, 2.0);
        assert_eq!([0.0, 1.0, 2.0].map(|value| half.normalize(value)), [0.0, 0.5, 1.0]);
    }

    /// A full circle without labels is as large as fits, centred; a half circle along the top
    /// is twice as wide as it is tall, so it uses the whole width of a wide area.
    #[test]
    fn the_circle_fits_its_arc() {
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(400.0, 200.0));

        let full = fit_circle(area, 0.0, 2.0 * PI, 0.0, &[]);
        assert!(close(full.center, Position::new(200.0, 100.0)));
        assert!((full.radius - 100.0).abs() < 1e-9);

        let top_half = fit_circle(area, -FRAC_PI_2, PI, 0.0, &[]);
        assert!((top_half.radius - 200.0).abs() < 1e-9, "the half circle's width is the area's: {}", top_half.radius);
        assert!(close(top_half.center, Position::new(200.0, 200.0)), "its centre sits on the area's lower edge: {:?}", top_half.center);
    }

    /// Labels around the rim shrink the circle until they fit, keeping it centred, but never
    /// below half its size without them.
    #[test]
    fn rim_labels_shrink_the_circle_to_fit_down_to_half() {
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(200.0, 200.0));
        let label = |angle: Scalar, width: Scalar| RimLabel { angle, size: Dimension::new(width, 10.0), gap: super::RIM_LABEL_GAP };

        let fitted = fit_circle(area, 0.0, 2.0 * PI, 0.0, &[label(0.0, 20.0), label(FRAC_PI_2, 30.0)]);
        // The right label needs 6 + 30 beside the rim, the top one 6 + 10 above it.
        assert!((fitted.radius - 64.0).abs() < 1e-9, "{}", fitted.radius);
        assert!(close(fitted.center, Position::new(100.0, 100.0)));

        let squeezed = fit_circle(area, 0.0, 2.0 * PI, 0.0, &[label(FRAC_PI_2, 500.0)]);
        assert!((squeezed.radius - 50.0).abs() < 1e-9, "never below half: {}", squeezed.radius);
    }

    /// A polygon ring crosses a spoke between two corners on the straight edge between them, and
    /// traced from one point to another, it turns at the corners in between.
    #[test]
    fn polygon_rings_run_straight_between_corners() {
        let polar = Polar::new(&angle(LinearScale::new(Axis::Radial)), &radius_scale(0.0), unit_circle());
        let corners = [0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2];
        let ring = Ring { polar: &polar, distance: 50.0, corners: Some(&corners) };

        assert!(close(ring.point(FRAC_PI_2), Position::new(150.0, 100.0)), "on a corner");
        assert!(close(ring.point(FRAC_PI_2 / 2.0), Position::new(125.0, 75.0)), "halfway along the edge from the top to the right");
        assert!(close(ring.point(-FRAC_PI_2 / 2.0), Position::new(75.0, 75.0)), "before the start, on the edge from the left");

        let mut traced = Traced(vec![]);
        ring.trace(&mut traced, FRAC_PI_2 / 2.0, PI + FRAC_PI_2 / 2.0);
        assert_eq!(traced.0.len(), 3, "the corners at the right and the bottom, then the end");
        assert!(close(traced.0[0], Position::new(150.0, 100.0)));
        assert!(close(traced.0[1], Position::new(100.0, 150.0)));
    }

    /// A ring's tangent runs around a circle, and along the edge on the side it heads to around a
    /// polygon, the edge it goes on to at a corner.
    #[test]
    fn rings_run_along_their_edges() {
        let polar = Polar::new(&angle(LinearScale::new(Axis::Radial)), &radius_scale(0.0), unit_circle());
        let circle = Ring { polar: &polar, distance: 50.0, corners: None };
        assert!(close(circle.tangent(0.0, 1.0), Position::new(1.0, 0.0)), "rightwards at the top, clockwise");
        assert!(close(circle.tangent(0.0, -1.0), Position::new(-1.0, 0.0)), "leftwards, the other way round");

        let corners = [0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2];
        let square = Ring { polar: &polar, distance: 50.0, corners: Some(&corners) };
        let down_right = Position::new(0.5f64.sqrt(), 0.5f64.sqrt());
        assert!(close(square.tangent(FRAC_PI_2 / 2.0, 1.0), down_right), "along the edge from the top to the right");
        assert!(close(square.tangent(0.0, 1.0), down_right), "at the top corner, the edge clockwise from it");
        assert!(close(square.tangent(0.0, -1.0), Position::new(-(0.5f64.sqrt()), 0.5f64.sqrt())), "and the edge counter-clockwise from it");
    }

    /// An arc is traced as curves of at most a quarter turn, ending on the circle where it
    /// should, in either direction.
    #[test]
    fn arcs_end_on_the_circle() {
        let center = Position::new(0.0, 0.0);
        let mut traced = Traced(vec![]);
        trace_arc(&mut traced, center, 10.0, 0.0, PI);
        assert_eq!(traced.0.len(), 6, "two quarter-turn curves of three points each");
        assert!(close(traced.0[2], Position::new(10.0, 0.0)), "a quarter turn clockwise is the right");
        assert!(close(traced.0[5], Position::new(0.0, 10.0)), "half a turn is the bottom");

        let mut back = Traced(vec![]);
        trace_arc(&mut back, center, 10.0, 0.0, -FRAC_PI_2);
        assert!(close(back.0[2], Position::new(-10.0, 0.0)), "counter-clockwise reaches the left");

        // A curve's midpoint lies on the circle, to well within a pixel.
        let [p0, c1, c2, p3] = [Position::new(0.0, -10.0), traced.0[0], traced.0[1], traced.0[2]];
        let mid = Position::new((p0.x + 3.0 * c1.x + 3.0 * c2.x + p3.x) / 8.0, (p0.y + 3.0 * c1.y + 3.0 * c2.y + p3.y) / 8.0);
        assert!(((mid.x * mid.x + mid.y * mid.y).sqrt() - 10.0).abs() < 0.01);
    }

    struct Traced(Vec<Position>);

    impl PathSink for Traced {
        fn move_to(&mut self, to: Position) {
            self.0.push(to);
        }

        fn line_to(&mut self, to: Position) {
            self.0.push(to);
        }

        fn bezier_curve_to(&mut self, control1: Position, control2: Position, to: Position) {
            self.0.extend([control1, control2, to]);
        }
    }

    /// `locate` and the normalized positions undo `position`, clockwise and counter-clockwise,
    /// from a turned start, with a hole in the middle.
    #[test]
    fn positions_map_back_to_angles_and_radii() {
        let scales = [
            angle(LinearScale::new(Axis::Radial)),
            angle(LinearScale::new(Axis::Radial).direction(AngleDirection::CounterClockwise)),
            angle(LinearScale::new(Axis::Radial).start_angle(Angle::Degrees(200.0))),
        ];

        for angle_scale in &scales {
            let polar = Polar::new(angle_scale, &radius_scale(0.3), unit_circle());
            for (a, r) in [(0.0, 0.0), (0.1, 1.0), (0.5, 0.25), (0.9, 0.75)] {
                let (back_a, back_r) = polar.normalized_at(polar.position(a, r));
                assert!((back_a - a).abs() < 1e-9 && (back_r - r).abs() < 1e-9, "{a}, {r} came back as {back_a}, {back_r}");
            }
        }

        let polar = Polar::new(&scales[0], &radius_scale(0.3), unit_circle());
        assert!(polar.normalized_distance(5.0) < 0.0, "in the hole");
        assert!(polar.normalized_distance(80.0) > 1.0, "outside the circle");
    }

    /// Outside a sweep of less than a full turn, an angle is taken to the nearer end: just below
    /// 0 before the start, just above 1 after the end.
    #[test]
    fn angles_outside_a_partial_sweep_go_to_the_nearer_end() {
        // The top half, from the left round to the right, clockwise.
        let half = angle(LinearScale::new(Axis::Radial).start_angle(Angle::Degrees(270.0)).end_angle(Angle::Degrees(450.0)));
        let polar = Polar::new(&half, &radius_scale(0.0), unit_circle());

        assert!((polar.normalized_angle(0.0) - 0.5).abs() < 1e-9, "the top is halfway");
        let after = polar.normalized_angle(FRAC_PI_2 + 0.2);
        assert!(after > 1.0 && after < 1.2, "just past the right: {after}");
        let before = polar.normalized_angle(-FRAC_PI_2 - 0.2);
        assert!(before < 0.0 && before > -0.2, "just before the left: {before}");
        let bottom = polar.normalized_angle(PI - 0.1);
        assert!(bottom > 1.0, "nearer the right than the left: {bottom}");
    }
}
