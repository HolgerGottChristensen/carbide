//! The geometry of bands between two lines: where the line through the highs crosses the line
//! through the lows, and the strips of the band between the crossings, so the parts where the
//! highs fall below the lows can be filled apart.
//!
//! The lines are traced with their interpolation, then flattened into polylines, fine enough that
//! the polygons filled from them match the curves stroked over them. Crossings are found by
//! sweeping along the categories every half pixel, which also handles curves, whose crossings have
//! no closed form.

use carbide::draw::{Position, Rect, Scalar};
use crate::controller::line_path::{trace_line, PathSink};
use crate::element::Interpolation;

/// How many straight pieces a curve between two points is flattened into.
const CURVE_STEPS: usize = 24;

/// How far apart, in pixels along the categories, the sweep compares the two lines.
const SWEEP_STEP: Scalar = 0.5;

/// The line through `points` as `trace_line` draws it, flattened into a polyline.
pub(crate) fn flatten(points: &[Position], interpolation: Interpolation, area: Rect) -> Vec<Position> {
    let mut flattened = Flattened(vec![]);
    trace_line(&mut flattened, points, interpolation, area);
    flattened.0
}

/// A strip of a band along the categories, from `start` to `end`, and whether the line through
/// the highs is below the line through the lows in it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Strip {
    pub(crate) start: Scalar,
    pub(crate) end: Scalar,
    pub(crate) inverted: bool,
}

/// The strips of the band between the polylines `highs` and `lows`, alternating between upright
/// and inverted, from where both lines start to where they end. `higher_is_greater` says which way
/// the values grow along y in the frame: a high is below its low where its y is the smaller with
/// it set, or the greater without. Empty when either line is.
pub(crate) fn strips(highs: &[Position], lows: &[Position], higher_is_greater: bool) -> Vec<Strip> {
    let (Some(start), Some(end)) = (span(highs, lows, Scalar::max, |p| p.first()), span(highs, lows, Scalar::min, |p| p.last())) else {
        return vec![];
    };

    if start > end {
        return vec![];
    }

    let inverted_at = |x: Scalar, high_cursor: &mut usize, low_cursor: &mut usize| {
        let (high, low) = (y_at(highs, x, high_cursor), y_at(lows, x, low_cursor));
        if higher_is_greater { high < low } else { high > low }
    };

    let (mut high_cursor, mut low_cursor) = (0, 0);
    let mut strips = vec![];
    let mut strip_start = start;
    let mut inverted = inverted_at(start, &mut high_cursor, &mut low_cursor);
    let mut x = start;

    while x < end {
        x = (x + SWEEP_STEP).min(end);
        let now = inverted_at(x, &mut high_cursor, &mut low_cursor);

        if now != inverted {
            strips.push(Strip { start: strip_start, end: x, inverted });
            strip_start = x;
            inverted = now;
        }
    }

    strips.push(Strip { start: strip_start, end, inverted });
    strips
}

/// The part of `polyline` from `start` to `end` along x, with points added where it crosses
/// either end.
pub(crate) fn clip(polyline: &[Position], start: Scalar, end: Scalar) -> Vec<Position> {
    let mut clipped = vec![];

    for (index, point) in polyline.iter().enumerate() {
        if let Some(previous) = index.checked_sub(1).map(|previous| polyline[previous]) {
            for edge in [start, end] {
                let crosses = (previous.x < edge && point.x > edge) || (previous.x > edge && point.x < edge);
                if crosses {
                    clipped.push(Position::new(edge, interpolate(previous, *point, edge)));
                }
            }
        }

        if point.x >= start && point.x <= end {
            clipped.push(*point);
        }
    }

    clipped
}

/// Where both lines have begun (`pick` = max of the first x values) or not yet ended (`pick` = min
/// of the last x values).
fn span(highs: &[Position], lows: &[Position], pick: fn(Scalar, Scalar) -> Scalar, end: fn(&[Position]) -> Option<&Position>) -> Option<Scalar> {
    Some(pick(end(highs)?.x, end(lows)?.x))
}

/// The polyline's y at `x`, moving `cursor` forwards to the piece holding `x`, as the sweep only
/// moves forwards. Where a piece is vertical, as at a step, the y after it.
fn y_at(polyline: &[Position], x: Scalar, cursor: &mut usize) -> Scalar {
    while *cursor + 1 < polyline.len() && polyline[*cursor + 1].x < x {
        *cursor += 1;
    }

    // Past vertical pieces at exactly `x`, to the y after the step.
    while *cursor + 2 < polyline.len() && polyline[*cursor + 1].x == x && polyline[*cursor + 2].x == x {
        *cursor += 1;
    }

    match (polyline.get(*cursor), polyline.get(*cursor + 1)) {
        (Some(from), Some(to)) => interpolate(*from, *to, x),
        (Some(only), None) => only.y,
        _ => 0.0,
    }
}

/// The y on the straight piece from `from` to `to` at `x`, or `to`'s y on a vertical piece.
fn interpolate(from: Position, to: Position, x: Scalar) -> Scalar {
    if to.x == from.x {
        return to.y;
    }

    let t = ((x - from.x) / (to.x - from.x)).clamp(0.0, 1.0);
    from.y + t * (to.y - from.y)
}

/// Records a traced line as a polyline, flattening curves.
struct Flattened(Vec<Position>);

impl PathSink for Flattened {
    fn move_to(&mut self, to: Position) {
        self.0.push(to);
    }

    fn line_to(&mut self, to: Position) {
        self.0.push(to);
    }

    fn bezier_curve_to(&mut self, control1: Position, control2: Position, to: Position) {
        let from = self.0.last().copied().unwrap_or(control1);

        for step in 1..=CURVE_STEPS {
            let t = step as Scalar / CURVE_STEPS as Scalar;
            let mt = 1.0 - t;
            let point = |a: Scalar, b: Scalar, c: Scalar, d: Scalar| mt * mt * mt * a + 3.0 * mt * mt * t * b + 3.0 * mt * t * t * c + t * t * t * d;
            self.0.push(Position::new(point(from.x, control1.x, control2.x, to.x), point(from.y, control1.y, control2.y, to.y)));
        }
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::{Dimension, Position, Rect};
    use crate::element::Interpolation;
    use super::{clip, flatten, strips, Strip};

    /// Two straight lines that cross in the middle: upright up to the crossing, inverted after it.
    /// The crossing is found within the sweep's half pixel.
    #[test]
    fn crossing_lines_make_an_upright_and_an_inverted_strip() {
        let highs = [Position::new(0.0, 10.0), Position::new(10.0, 0.0)];
        let lows = [Position::new(0.0, 0.0), Position::new(10.0, 10.0)];

        let found = strips(&highs, &lows, true);

        assert_eq!(found.len(), 2, "{found:?}");
        assert!(!found[0].inverted && found[1].inverted);
        assert!((found[0].end - 5.0).abs() <= 0.5, "the crossing is at 5, found at {}", found[0].end);
        assert_eq!((found[0].start, found[1].end), (0.0, 10.0));
    }

    /// With values growing downwards along y, as on the screen, the same lines are the other way
    /// round: inverted first, upright after the crossing. Lines that never cross make one strip.
    #[test]
    fn which_side_is_high_follows_the_frame() {
        let highs = [Position::new(0.0, 10.0), Position::new(10.0, 0.0)];
        let lows = [Position::new(0.0, 0.0), Position::new(10.0, 10.0)];

        let found = strips(&highs, &lows, false);
        assert!(found[0].inverted && !found[1].inverted);

        let apart = strips(&[Position::new(0.0, 5.0), Position::new(10.0, 5.0)], &[Position::new(0.0, 1.0), Position::new(10.0, 1.0)], true);
        assert_eq!(apart, vec![Strip { start: 0.0, end: 10.0, inverted: false }]);
    }

    /// Clipping a polyline to a strip keeps the points inside it, with points added where it
    /// crosses the strip's ends.
    #[test]
    fn clipping_adds_points_at_the_strip_ends() {
        let line = [Position::new(0.0, 0.0), Position::new(4.0, 8.0), Position::new(10.0, 2.0)];

        assert_eq!(clip(&line, 2.0, 7.0), vec![Position::new(2.0, 4.0), Position::new(4.0, 8.0), Position::new(7.0, 5.0)]);
    }

    /// A curve is flattened into many points that start and end at the curve's ends; straight
    /// lines keep their points.
    #[test]
    fn curves_are_flattened() {
        let area = Rect::new(Position::new(0.0, 0.0), Dimension::new(100.0, 100.0));
        let points = [Position::new(0.0, 50.0), Position::new(50.0, 10.0), Position::new(100.0, 60.0)];

        let curve = flatten(&points, Interpolation::Monotone, area);
        assert!(curve.len() > 10, "{}", curve.len());
        assert_eq!((curve[0], *curve.last().unwrap()), (points[0], points[2]));

        assert_eq!(flatten(&points, Interpolation::Linear, area), points.to_vec());
    }
}
