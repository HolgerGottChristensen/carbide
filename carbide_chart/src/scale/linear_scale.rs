use std::ops::{Range, RangeInclusive};
use carbide::draw::{Dimension, Position, Scalar};
use carbide::environment::Environment;
use carbide::widget::canvas::CanvasContext;
use crate::scale::{Axis, Scale};
use crate::scale::bounds::Bounds;
use crate::scale::percent_or_value::PercentOrValue;

const MIN_TICK_SPACING: f64 = 1e-14;

#[derive(Clone, Debug)]
pub struct LinearScale {
    axis: Axis,
    min: Option<Scalar>,
    max: Option<Scalar>,
    range: Range<Scalar>,
    nice_range: Range<Scalar>,
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
            min: None,
            max: None,
            range: 0.0..1.0,
            nice_range: 0.0..1.0,
            begin_at_zero: false,
            grace: PercentOrValue::None,
            include_bounds: true,
            step_size: None,
            precision: None,
            bounds: Bounds::Ticks,
        }
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
                (self.range.end.abs() * 0.05)
            };

            self.range.end = self.range.end + offset;

            if !self.begin_at_zero {
                self.range.start = self.range.start - offset;
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

    fn min(&self) -> Scalar {
        self.range.start
    }

    fn max(&self) -> Scalar {
        self.range.end
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
    }

    fn display_ticks(&self) -> bool {
        true
    }

    fn display_grid(&self) -> bool {
        true
    }

    fn ticks(&self) -> (Vec<Scalar>, Range<Scalar>) {
        let range_min = self.range.start;
        let range_max = self.range.end;
        let min = self.min;
        let max = self.max;

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
            return (vec![range_min, range_max], range_min..range_max);
        }

        let mut num_spaces = (f64::ceil(range_max / spacing) - f64::floor(range_min / spacing)) as u32;

        if num_spaces > max_spaces {
            // If the calculated num of spaces exceeds maxNumSpaces, recalculate it
            spacing = nice_num(num_spaces as Scalar * spacing / max_spaces as Scalar / unit) * unit;
        }

        let mut factor: Scalar;

        if let Some(precision) = self.precision {
            factor = 10i64.pow(precision as u32) as Scalar;
            spacing = (spacing * factor).ceil() / factor;
        }

        let mut nice_min: f64;
        let mut nice_max: f64;

        if self.bounds == Bounds::Ticks {
            nice_min = (range_min / spacing).floor() * spacing;
            nice_max = (range_max / spacing).ceil() * spacing;
        } else {
            nice_min = range_min;
            nice_max = range_max;
        }


        let mut num_spaces_float = (nice_max - nice_min) / spacing;

        // If very close to our rounded value, use it.
        if almost_equals(num_spaces_float, num_spaces_float.round(), spacing / 1000.0) {
            num_spaces = num_spaces_float.round() as u32;
        } else {
            num_spaces = num_spaces_float.ceil() as u32;
        }

        if let Some(precision) = self.precision {
            factor = 10u32.pow(precision as u32) as Scalar;
        } else {
            let decimal_places = u32::max(decimal_places(spacing), decimal_places(nice_min)).min(10);
            factor = 10u32.pow(decimal_places) as Scalar;
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

            if let Some(max) = self.max {
                if tick_value > max {
                    break;
                }
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

        (ticks, nice_min..nice_max)
    }
}

fn nice_num(range: Scalar) -> Scalar {
    let roundedRange = range.round();

    let almost = almost_equals(range, roundedRange, range / 1000.0);

    let range = if almost { roundedRange } else { range };

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

    return nice_fraction * nice_range;
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