use std::ops::Range;

use carbide::draw::Scalar;

use crate::scale::{Axis, AxisPosition, Scale};
use crate::scale::bounds::Bounds;
use crate::scale::percent_or_value::PercentOrValue;

const MIN_TICK_SPACING: f64 = 1e-14;

#[derive(Clone, Debug)]
pub struct LinearScale {
    axis: Axis,
    position: AxisPosition,
    weight: i32,
    grid_lines: bool,
    tick_marks: bool,
    offset: bool,
    min: Option<Scalar>,
    max: Option<Scalar>,
    range: Range<Scalar>,
    nice_range: Range<Scalar>,
    ticks: Vec<Scalar>,
    begin_at_zero: bool,
    grace: PercentOrValue,
    include_bounds: bool,
    step_size: Option<f64>,
    precision: Option<i8>,
    bounds: Bounds,
}

impl LinearScale {
    pub fn new(axis: Axis) -> LinearScale {
        LinearScale {
            axis,
            position: axis.default_position(),
            weight: 0,
            grid_lines: true,
            tick_marks: true,
            offset: false,
            min: None,
            max: None,
            range: 0.0..1.0,
            nice_range: 0.0..1.0,
            ticks: vec![],
            begin_at_zero: false,
            grace: PercentOrValue::None,
            include_bounds: true,
            step_size: None,
            precision: None,
            bounds: Bounds::Ticks,
        }
    }

    /// Draws the scale on the given side of the chart area. Horizontal scales go above or below
    /// it, vertical scales left or right of it; any other side is logged and the scale keeps its
    /// current position.
    pub fn position(mut self, position: AxisPosition) -> LinearScale {
        if self.axis.allows(position) {
            self.position = position;
        } else {
            log::warn!("A {:?} scale can't be positioned {:?}; keeping {:?}", self.axis, position, self.position);
        }

        self
    }

    /// Orders the scale among the scales on the same side: a higher weight is placed further
    /// away from the chart area. Defaults to 0. See `Scale::weight`.
    pub fn weight(mut self, weight: i32) -> LinearScale {
        self.weight = weight;
        self
    }

    /// Whether the scale draws grid lines across the chart area. Defaults to `true`. Turn it off
    /// for secondary scales so their lines don't interleave with the primary scale's, like
    /// Chart.js's `grid.drawOnChartArea: false`; the labels stay.
    pub fn grid_lines(mut self, grid_lines: bool) -> LinearScale {
        self.grid_lines = grid_lines;
        self
    }

    /// Whether the scale draws tick marks in its band, between the chart area and its labels.
    /// Defaults to `true`, like Chart.js's `grid.drawTicks`.
    pub fn tick_marks(mut self, tick_marks: bool) -> LinearScale {
        self.tick_marks = tick_marks;
        self
    }

    fn tick_limits(&self) -> u32 {
        11
    }

    fn calculate_begin_zero(&mut self) {
        if self.begin_at_zero {
            // If both are negative, we set the max to 0.0, otherwise we set min to 0.0
            if self.range.start.is_sign_negative() && self.range.end.is_sign_negative() {
                self.range.end = 0.0;
            } else {
                self.range.start = 0.0;
            }
        }

        if self.range.start == self.range.end {
            let offset = if self.range.end == 0.0 {
                1.0
            } else {
                self.range.end.abs() * 0.05
            };

            self.range.end += offset;

            if !self.begin_at_zero {
                self.range.start -= offset;
            }
        }
    }

    fn calculate_grace(&mut self) {
        // Add additional grace to the scale, in both directions, unless we need the start or end is zero
        let change = match self.grace {
            PercentOrValue::Percent(percent) => { percent * (self.range.end - self.range.start) / 2.0 }
            PercentOrValue::Value(value) => { value }
            PercentOrValue::None => { 0.0 }
        };

        if !self.begin_at_zero || self.range.start != 0.0 {
            self.range.start -= change.abs();
        }

        if !self.begin_at_zero || self.range.end != 0.0 {
            self.range.end += change.abs();
        }
    }
}

impl Scale for LinearScale {
    fn axis(&self) -> Axis {
        self.axis
    }

    fn axis_position(&self) -> AxisPosition {
        self.position
    }

    fn weight(&self) -> i32 {
        self.weight
    }

    /// With an offset, the mapped range is widened by half a tick step at both ends, as in
    /// Chart.js's `LinearScaleBase::configure`.
    fn normalize(&self, value: Scalar) -> Scalar {
        let mut start = self.nice_range.start;
        let mut end = self.nice_range.end;

        if self.offset && !self.ticks.is_empty() {
            let half_step = (end - start) / (self.ticks.len() - 1).max(1) as Scalar / 2.0;
            start -= half_step;
            end += half_step;
        }

        (value - start) / (end - start)
    }

    fn set_offset(&mut self, offset: bool) {
        self.offset = offset;
    }

    fn min(&self) -> Scalar {
        self.nice_range.start
    }

    fn max(&self) -> Scalar {
        self.nice_range.end
    }

    fn set_range(&mut self, min: Scalar, max: Scalar) {
        if min.is_finite() {
            self.range.start = min;
        } else {
            self.range.start = 0.0;
        }

        if max.is_finite() {
            self.range.end = max;
        } else {
            self.range.end = 1.0;
        }

        self.calculate_begin_zero();

        self.calculate_grace();
        self.recalculate_ticks();
    }

    fn display_ticks(&self) -> bool {
        self.tick_marks
    }

    fn display_grid(&self) -> bool {
        self.grid_lines
    }

    fn range(&self) -> Range<Scalar> {
        self.nice_range.clone()
    }

    fn recalculate_ticks(&mut self) {
        let range_min = self.range.start;
        let range_max = self.range.end;
        let _min = self.min;
        let _max = self.max;

        let max_ticks = self.tick_limits().max(2);
        let mut ticks = vec![];

        // We have one less space between ticks than ticks
        let max_spaces = max_ticks - 1;
        //let max_digits = 3;
        let unit = 1 as Scalar;
        //let min_spacing = (self.max - self.min) / (max_digits + 1) as Scalar;

        let mut spacing = nice_num((range_max - range_min) / max_spaces as Scalar / unit) * unit;

        // Beyond MIN_TICK_SPACING floating point numbers being to lose precision
        // such that we can't do the math necessary to generate ticks
        if spacing <= MIN_TICK_SPACING {
            self.nice_range = range_min..range_max;
            self.ticks = vec![range_min, range_max];
            return;
        }

        let mut num_spaces = (steps_at_or_above(range_max, spacing) - steps_at_or_below(range_min, spacing)) as u32;

        if num_spaces > max_spaces {
            // If the calculated num of spaces exceeds maxNumSpaces, recalculate it
            spacing = nice_num(num_spaces as Scalar * spacing / max_spaces as Scalar / unit) * unit;
        }

        let mut factor: Scalar;

        if let Some(precision) = self.precision {
            factor = rounding_factor(precision as u32);
            spacing = (spacing * factor).ceil() / factor;
        }

        let mut nice_min: f64;
        let mut nice_max: f64;

        if self.bounds == Bounds::Ticks {
            nice_min = steps_at_or_below(range_min, spacing) * spacing;
            nice_max = steps_at_or_above(range_max, spacing) * spacing;
        } else {
            nice_min = range_min;
            nice_max = range_max;
        }


        let num_spaces_float = (nice_max - nice_min) / spacing;

        // If very close to our rounded value, use it.
        if almost_equals(num_spaces_float, num_spaces_float.round(), spacing / 1000.0) {
            num_spaces = num_spaces_float.round() as u32;
        } else {
            num_spaces = num_spaces_float.ceil() as u32;
        }

        if let Some(precision) = self.precision {
            factor = rounding_factor(precision as u32);
        } else {
            let decimal_places = u32::max(decimal_places(spacing), decimal_places(nice_min)).min(10);
            factor = rounding_factor(decimal_places);
        }

        nice_min = (nice_min * factor).round() / factor;
        nice_max = (nice_max * factor).round() / factor;

        let mut j = 0;

        if let Some(min) = self.min {
            if self.include_bounds && nice_min != min {
                ticks.push(min);

                if nice_min < min {
                    j += 1;
                }
            } else {
                j += 1;
            }
        }

        for k in j..num_spaces as i32 {
            let tick_value = ((nice_min + k as Scalar * spacing) * factor) / factor;

            if let Some(max) = self.max
                && tick_value > max {
                    break;
                }

            ticks.push(tick_value);
        }

        if let Some(max) = self.max {
            if self.include_bounds && nice_max != max {
                todo!()
            } else if nice_max == max {
                ticks.push(nice_max);
            }
        } else {
            ticks.push(nice_max);
        }

        self.nice_range = nice_min..nice_max;
        self.ticks = ticks;
    }

    fn ticks(&self) -> &[Scalar] {
        &self.ticks
    }
}

fn nice_num(range: Scalar) -> Scalar {
    let rounded_range = range.round();

    let almost = almost_equals(range, rounded_range, range / 1000.0);

    let range = if almost { rounded_range } else { range };

    let nice_range = f64::powi(10.0, range.log10().floor() as i32);
    let fraction = range / nice_range;

    let nice_fraction = if fraction <= 1.0 {
        1.0
    } else if fraction <= 2.0 {
        2.0
    } else if fraction <= 5.0 {
        5.0
    } else {
        10.0
    };

    nice_fraction * nice_range
}

/// `10^decimals`, the factor ticks are rounded with: `(value * factor).round() / factor` keeps
/// `decimals` decimal places. A float, as Chart.js's `Math.pow(10, …)` in `generateTicks`; it
/// used to be a `u32`, which overflows from 10 decimals, reached when floating-point noise makes
/// a tick like 0.9500000000000001 look like it has many.
fn rounding_factor(decimals: u32) -> Scalar {
    Scalar::powi(10.0, decimals as i32)
}

/// The largest whole number of `step`s at or below `x`: `floor(x / step)`, except that a quotient
/// within floating-point noise of a whole number counts as that number. Without this, a value
/// already on a step can come out a step short: 1.9 / 0.05 is 37.99999999999999, which floors to
/// 37, so a scale meant to start at 1.9 started at 1.85. matplotlib's `_Edge_integer.le`
/// (`lib/matplotlib/ticker.py`) does the same; Chart.js floors plainly (`generateTicks` in
/// `src/scales/scale.linearbase.js`).
fn steps_at_or_below(x: Scalar, step: Scalar) -> Scalar {
    let quotient = x / step;
    let whole = quotient.round();

    if almost_equals(quotient, whole, step_tolerance(quotient)) { whole } else { quotient.floor() }
}

/// The smallest whole number of `step`s at or above `x`: `ceil(x / step)`, with the same snapping
/// as `steps_at_or_below` (matplotlib's `_Edge_integer.ge`).
fn steps_at_or_above(x: Scalar, step: Scalar) -> Scalar {
    let quotient = x / step;
    let whole = quotient.round();

    if almost_equals(quotient, whole, step_tolerance(quotient)) { whole } else { quotient.ceil() }
}

/// How close to a whole number a quotient must be to count as one. Floating-point error grows
/// with the size of the numbers, so the tolerance does too, from 1e-10, as in matplotlib's
/// `_Edge_integer.closeto`; it stays under half a step, so it never snaps to the wrong neighbour.
fn step_tolerance(quotient: Scalar) -> Scalar {
    (quotient.abs() * 1e-12).clamp(1e-10, 0.4999)
}

fn almost_equals(x: Scalar, y: Scalar, epsilon: Scalar) -> bool {
    (x - y).abs() < epsilon
}

fn decimal_places(x: Scalar) -> u32 {
    if x.is_infinite() {
        return 0;
    }

    let mut e = 1.0;
    let mut p = 0;

    while (x * e).round() / e != x {
        e *= 10.0;
        p += 1;
    }

    p
}
#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;
    use crate::scale::{Axis, LinearScale, Scale};

    /// A range where min equals max, as from a dataset of one point or of equal values, is
    /// widened by 5% of the value (or by 1 at zero), as Chart.js's `handleTickRangeOptions`
    /// does, and gets ticks around the value. It used to panic: rounding the ticks computed
    /// `10u32.pow(10)`, which overflows, for a range like 0.95..1.05.
    #[test]
    fn equal_min_and_max_get_a_range_around_the_value() {
        for value in [1.0, 0.0, -3.0, 0.1, 1234.5, 1e-7] {
            let mut scale = LinearScale::new(Axis::Vertical);
            scale.set_range(value, value);

            let ticks = scale.ticks();
            assert!(ticks.len() >= 2, "{value}: expected ticks around the value, got {ticks:?}");
            assert!(scale.min() < value && value < scale.max(), "{value}: the value should be inside {}..{}", scale.min(), scale.max());

            let normalized = scale.normalize(value);
            assert!(normalized.is_finite() && normalized > 0.0 && normalized < 1.0, "{value}: normalized to {normalized}");
        }
    }

    /// A single point at 2 is widened to 1.9..2.1 with a step of 0.05. 1.9 / 0.05 is
    /// 37.99999999999999 in floating point, which used to floor to 37 and start the scale a step
    /// early, at 1.85, so the point wasn't centred.
    #[test]
    fn a_range_already_on_a_step_is_not_widened_by_noise() {
        let mut scale = LinearScale::new(Axis::Horizontal);
        scale.set_range(2.0, 2.0);

        assert!((scale.min() - 1.9).abs() < 1e-12, "the scale should start at 1.9, not {}", scale.min());
        assert!((scale.max() - 2.1).abs() < 1e-12, "the scale should end at 2.1, not {}", scale.max());
        assert!((scale.normalize(2.0) - 0.5).abs() < 1e-9, "the point should be centred");
        assert!((scale.ticks()[0] - 1.9).abs() < 1e-12, "the first tick should be 1.9, not {}", scale.ticks()[0]);
    }

    /// Ticks whose rounding needs ten or more decimals: the factor is a float, as in Chart.js,
    /// so it can't overflow.
    #[test]
    fn ticks_with_many_decimals_do_not_overflow() {
        let mut scale = LinearScale::new(Axis::Vertical);
        scale.set_range(0.95, 1.05);
        assert!(scale.ticks().len() >= 2);

        let mut scale = LinearScale::new(Axis::Vertical);
        scale.set_range(1e-9, 3e-9);
        assert!(scale.ticks().iter().all(|tick: &Scalar| tick.is_finite()));
    }
}
