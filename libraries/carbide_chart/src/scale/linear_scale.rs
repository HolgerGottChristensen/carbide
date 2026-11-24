use std::ops::Range;

use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::{AnyReadState, IntoReadState, ReadStateExt};

use crate::scale::{Axis, AxisOptions, AxisPosition, PolarRole, MIN_ZOOM_SPAN};

use crate::Scale;

const MIN_TICK_SPACING: f64 = 1e-14;

#[derive(Clone, Debug)]
pub struct LinearScale {
    axis: Axis,
    axis_options: AxisOptions,
    offset: bool,
    range: Range<Scalar>,
    nice_range: Range<Scalar>,
    ticks: Vec<Scalar>,
    /// The ticks in the zoom window, when the scale shows only part of its range (see `zoom`).
    visible_ticks: Option<Vec<Scalar>>,
    /// Where the scale starts, set with `min`, in place of its fitted start.
    fixed_min: Option<Box<dyn AnyReadState<T=Scalar>>>,
    /// Where the scale ends, set with `max`, in place of its fitted end.
    fixed_max: Option<Box<dyn AnyReadState<T=Scalar>>>,
    /// The distance between ticks, set with `step`, in place of a nice one picked for the range.
    fixed_step: Option<Box<dyn AnyReadState<T=Scalar>>>,
}

impl LinearScale {
    pub fn new(axis: Axis) -> LinearScale {
        LinearScale {
            axis,
            axis_options: AxisOptions::new(axis),
            offset: false,
            range: 0.0..1.0,
            nice_range: 0.0..1.0,
            ticks: vec![],
            visible_ticks: None,
            fixed_min: None,
            fixed_max: None,
            fixed_step: None,
        }
    }

    /// Starts the scale at `min`, a number or a state of one, whatever the data: exactly there,
    /// not rounded to a tick. Values below it fall outside the chart area.
    /// Without it, the scale starts at the smallest value, rounded down to a whole tick step.
    ///
    /// This is a builder method: on a `LinearScale` value, it takes the place of `Scale::min`,
    /// which reads where the scale starts after a fit; call that as `Scale::min(&scale)`.
    pub fn min(mut self, min: impl IntoReadState<Scalar>) -> LinearScale {
        self.fixed_min = Some(min.into_read_state().as_dyn_read());
        self
    }

    /// Ends the scale at `max`, a number or a state of one, whatever the data, as `min` starts
    /// it. An angle in degrees spans a whole circle with `.min(0.0).max(360.0)`.
    pub fn max(mut self, max: impl IntoReadState<Scalar>) -> LinearScale {
        self.fixed_max = Some(max.into_read_state().as_dyn_read());
        self
    }

    /// Puts the ticks `step` apart, a number or a state of one, on whole multiples of it,
    /// instead of a nice step picked for the range: `.step(45.0)` on an angle in degrees. A
    /// step too small for the range, giving more than 1000 ticks, or not above zero, is ignored.
    pub fn step(mut self, step: impl IntoReadState<Scalar>) -> LinearScale {
        self.fixed_step = Some(step.into_read_state().as_dyn_read());
        self
    }

    /// The fixed start, end and step, as their states were last synced, leaving out values that
    /// aren't finite.
    fn fixed(&self) -> (Option<Scalar>, Option<Scalar>, Option<Scalar>) {
        let read = |state: &Option<Box<dyn AnyReadState<T=Scalar>>>| state.as_ref().map(|state| *state.value_dyn()).filter(|value| value.is_finite());
        (read(&self.fixed_min), read(&self.fixed_max), read(&self.fixed_step).filter(|step| *step > 0.0))
    }

    /// Draws the scale on the given side of the chart area, a side or a state of one. Horizontal
    /// scales go above or below it, vertical scales left or right of it; any other side is logged
    /// and the scale is drawn on its default side.
    pub fn position(mut self, position: impl IntoReadState<AxisPosition>) -> LinearScale {
        self.axis_options = self.axis_options.with_position(position);
        self
    }

    /// Orders the scale among the scales on the same side: a higher weight is placed further
    /// away from the chart area. Defaults to 0. A value or a state of one. See
    /// `AxisOptions::weight`.
    pub fn weight(mut self, weight: impl IntoReadState<i32>) -> LinearScale {
        self.axis_options = self.axis_options.with_weight(weight);
        self
    }

    /// Whether the scale draws grid lines across the chart area. Defaults to `true`. Turn it off
    /// for secondary scales so their lines don't interleave with the primary scale's; the labels
    /// stay. A value or a state of one.
    pub fn grid_lines(mut self, grid_lines: impl IntoReadState<bool>) -> LinearScale {
        self.axis_options = self.axis_options.with_grid_lines(grid_lines);
        self
    }

    /// Whether the scale draws tick marks in its band, between the chart area and its labels.
    /// Defaults to `true`. A value or a state of one.
    pub fn tick_marks(mut self, tick_marks: impl IntoReadState<bool>) -> LinearScale {
        self.axis_options = self.axis_options.with_tick_marks(tick_marks);
        self
    }

    /// Gives the scale's axis a title, a text or a state of one, drawn centred along the axis
    /// beyond its labels. A vertical axis turns it along the axis: on the left it reads from
    /// bottom to top, on the right from top to bottom.
    pub fn title(mut self, title: impl IntoReadState<String>) -> LinearScale {
        self.axis_options = self.axis_options.with_title(title);
        self
    }

    crate::scale::axis_options::polar_options!(LinearScale);
    crate::scale::axis_options::zoom_options!(LinearScale);
    crate::scale::axis_options::label_options!(LinearScale);

    /// At most how many ticks the scale has: fewer along a radius, which is half as long as the
    /// chart area is wide, and whose labels sit beside one spoke.
    fn tick_limits(&self) -> u32 {
        match self.axis_options.polar_role() {
            Some(PolarRole::Radius) => 6,
            _ => 11,
        }
    }

    /// Widens a range with no width, as from a single point or equal values, by 5% of the value
    /// at both ends, or by 1 at zero, so the value sits inside the scale with ticks around it. With
    /// one end fixed (see `min` and `max`), only the other end moves, by twice as much.
    fn widen_empty_range(&mut self) {
        if self.range.start == self.range.end {
            let offset = if self.range.end == 0.0 {
                1.0
            } else {
                self.range.end.abs() * 0.05
            };

            // A fixed end stays where it was set; the free end moves instead, by twice as much.
            let (fixed_min, fixed_max, _) = self.fixed();
            match (fixed_min.is_some(), fixed_max.is_some()) {
                (true, false) => self.range.end += 2.0 * offset,
                (false, true) => self.range.start -= 2.0 * offset,
                _ => {
                    self.range.end += offset;
                    self.range.start -= offset;
                }
            }
        }
    }
}

impl Scale for LinearScale {
    fn axis(&self) -> Axis {
        self.axis
    }

    fn axis_options(&self) -> &AxisOptions {
        &self.axis_options
    }

    fn axis_options_mut(&mut self) -> Option<&mut AxisOptions> {
        Some(&mut self.axis_options)
    }

    fn set_polar_role(&mut self, role: Option<crate::scale::PolarRole>) {
        self.axis_options.set_polar_role(role);
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        let mut changed = self.axis_options.sync(env);
        for state in [&mut self.fixed_min, &mut self.fixed_max, &mut self.fixed_step].into_iter().flatten() {
            changed |= state.sync(env);
        }
        changed
    }

    /// With an offset, the mapped range is widened by half a tick step at both ends; zoomed, the
    /// window of it is mapped onto the chart area.
    fn normalize(&self, value: Scalar) -> Scalar {
        let (start, end) = self.mapped_range();
        self.axis_options.window().zoom((value - start) / (end - start))
    }

    fn denormalize(&self, normalized: Scalar) -> Scalar {
        let (start, end) = self.mapped_range();
        start + self.axis_options.window().unzoom(normalized) * (end - start)
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

        // A fixed end replaces the data's; one fixed end past the other data end moves that too,
        // so the range never turns over.
        let (fixed_min, fixed_max, _) = self.fixed();
        if let Some(min) = fixed_min {
            self.range.start = min;
            self.range.end = self.range.end.max(min);
        }
        if let Some(max) = fixed_max {
            self.range.end = max;
            self.range.start = self.range.start.min(max);
        }

        self.widen_empty_range();
        self.recalculate_ticks();
    }

    fn range(&self) -> Range<Scalar> {
        self.nice_range.clone()
    }

    /// Picks a "nice" spacing (1, 2 or 5 times a power of ten) giving at most `tick_limits`
    /// ticks, and rounds the range out to whole steps. With a fixed start, end or step (see
    /// `min`, `max` and `step`), those are kept instead (see `fixed_ticks`). Zoomed, the ticks
    /// shown are picked the same way for the part of the range in the window (see `zoom`).
    fn recalculate_ticks(&mut self) {
        let (fixed_min, fixed_max, fixed_step) = self.fixed();
        if fixed_min.is_some() || fixed_max.is_some() || fixed_step.is_some() {
            self.fixed_ticks(fixed_min.is_some(), fixed_max.is_some(), fixed_step);
        } else {
            let (nice_range, ticks) = nice_ticks(self.range.start, self.range.end, self.tick_limits().max(2) - 1);
            self.nice_range = nice_range;
            self.ticks = ticks;
        }

        self.zoom_ticks();
    }

    fn ticks(&self) -> &[Scalar] {
        self.visible_ticks.as_deref().unwrap_or(&self.ticks)
    }
}

impl LinearScale {
    /// Decides the window the scale shows from the zoom asked for, and, when it shows only part
    /// of the range, the ticks for that part: nice ones for it, as for a fit, but only those
    /// inside it, or the fixed step's within it.
    fn zoom_ticks(&mut self) {
        let window = self.axis_options.zoom().limited(MIN_ZOOM_SPAN);
        self.axis_options.set_window(window);
        self.visible_ticks = None;

        if window.is_whole() {
            return;
        }

        let (start, end) = self.mapped_range();
        let (low, high) = (start + window.start * (end - start), start + window.end * (end - start));
        let max_spaces = self.tick_limits().max(2) - 1;

        let ticks = match self.fixed().2 {
            Some(step) if step > MIN_TICK_SPACING => {
                let first = steps_at_or_above(low, step);
                let last = steps_at_or_below(high, step);
                (first as i64..=last as i64).map(|k| k as Scalar * step).collect()
            }
            _ => nice_ticks(low, high, max_spaces).1,
        };

        let tolerance = (high - low) * 1e-9;
        self.visible_ticks = Some(ticks.into_iter().filter(|tick| *tick >= low - tolerance && *tick <= high + tolerance).collect());
    }
}

/// Nice ticks for the range from `range_min` to `range_max`: a spacing of 1, 2 or 5 times a power
/// of ten giving at most `max_spaces` spaces, and the range rounded out to whole steps.
fn nice_ticks(range_min: Scalar, range_max: Scalar, max_spaces: u32) -> (Range<Scalar>, Vec<Scalar>) {
    let mut spacing = nice_num((range_max - range_min) / max_spaces as Scalar);

    // Beyond MIN_TICK_SPACING floating point numbers being to lose precision
    // such that we can't do the math necessary to generate ticks
    if spacing <= MIN_TICK_SPACING {
        return (range_min..range_max, vec![range_min, range_max]);
    }

    let num_spaces = (steps_at_or_above(range_max, spacing) - steps_at_or_below(range_min, spacing)) as u32;

    if num_spaces > max_spaces {
        // If the calculated num of spaces exceeds maxNumSpaces, recalculate it
        spacing = nice_num(num_spaces as Scalar * spacing / max_spaces as Scalar);
    }

    let mut nice_min = steps_at_or_below(range_min, spacing) * spacing;
    let mut nice_max = steps_at_or_above(range_max, spacing) * spacing;

    let num_spaces_float = (nice_max - nice_min) / spacing;

    // If very close to our rounded value, use it.
    let num_spaces = if almost_equals(num_spaces_float, num_spaces_float.round(), spacing / 1000.0) {
        num_spaces_float.round() as u32
    } else {
        num_spaces_float.ceil() as u32
    };

    let decimal_places = u32::max(decimal_places(spacing), decimal_places(nice_min)).min(10);
    let factor = rounding_factor(decimal_places);

    nice_min = (nice_min * factor).round() / factor;
    nice_max = (nice_max * factor).round() / factor;

    let mut ticks: Vec<Scalar> = (0..num_spaces)
        .map(|k| ((nice_min + k as Scalar * spacing) * factor) / factor)
        .collect();
    ticks.push(nice_max);

    (nice_min..nice_max, ticks)
}

impl LinearScale {
    /// The values at the start and end of the chart area: the nice range, padded by half a step at
    /// each end with an offset (see `Scale::set_offset`).
    fn mapped_range(&self) -> (Scalar, Scalar) {
        let (mut start, mut end) = (self.nice_range.start, self.nice_range.end);

        if self.offset && !self.ticks.is_empty() {
            let half_step = (end - start) / (self.ticks.len() - 1).max(1) as Scalar / 2.0;
            start -= half_step;
            end += half_step;
        }

        (start, end)
    }

    /// The ticks for a range with a fixed start, end or step. The step is the fixed one, or a nice
    /// one for the range, as without; an end that isn't fixed is rounded out to a whole step, and
    /// a fixed end is kept exactly, with a tick of its own. In between, ticks fall on whole
    /// multiples of the step, so `.min(3.0)` with a step of 10 has ticks at 3, 10, 20, ...
    fn fixed_ticks(&mut self, min_fixed: bool, max_fixed: bool, step: Option<Scalar>) {
        let (start, end) = (self.range.start, self.range.end);

        let max_spaces = self.tick_limits().max(2) - 1;
        let step = step
            .filter(|step| (end - start) / step <= 1000.0)
            .unwrap_or_else(|| nice_num((end - start) / max_spaces as Scalar).max(MIN_TICK_SPACING));

        let nice_min = if min_fixed { start } else { steps_at_or_below(start, step) * step };
        let nice_max = if max_fixed { end } else { steps_at_or_above(end, step) * step };

        let decimal_places = u32::max(decimal_places(step), decimal_places(nice_min)).min(10);
        let factor = rounding_factor(decimal_places);
        let tolerance = step / 1000.0;

        let mut ticks = vec![nice_min];
        let mut k = steps_at_or_below(nice_min, step) + 1.0;
        loop {
            let tick = (k * step * factor).round() / factor;
            if tick >= nice_max - tolerance {
                break;
            }
            if tick > nice_min + tolerance {
                ticks.push(tick);
            }
            k += 1.0;
        }
        if nice_max > nice_min + tolerance {
            ticks.push(nice_max);
        }

        self.nice_range = nice_min..nice_max;
        self.ticks = ticks;
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
/// `decimals` decimal places. A float: it used to be a `u32`, which overflows from 10 decimals, reached when floating-point noise makes
/// a tick like 0.9500000000000001 look like it has many.
fn rounding_factor(decimals: u32) -> Scalar {
    Scalar::powi(10.0, decimals as i32)
}

/// The largest whole number of `step`s at or below `x`: `floor(x / step)`, except that a quotient
/// within floating-point noise of a whole number counts as that number. Without this, a value
/// already on a step can come out a step short: 1.9 / 0.05 is 37.99999999999999, which floors to
/// 37, so a scale meant to start at 1.9 started at 1.85.
fn steps_at_or_below(x: Scalar, step: Scalar) -> Scalar {
    let quotient = x / step;
    let whole = quotient.round();

    if almost_equals(quotient, whole, step_tolerance(quotient)) { whole } else { quotient.floor() }
}

/// The smallest whole number of `step`s at or above `x`: `ceil(x / step)`, with the same snapping
/// as `steps_at_or_below`.
fn steps_at_or_above(x: Scalar, step: Scalar) -> Scalar {
    let quotient = x / step;
    let whole = quotient.round();

    if almost_equals(quotient, whole, step_tolerance(quotient)) { whole } else { quotient.ceil() }
}

/// How close to a whole number a quotient must be to count as one. Floating-point error grows
/// with the size of the numbers, so the tolerance does too, from 1e-10; it stays under half a
/// step, so it never snaps to the wrong neighbour.
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
    use carbide::environment::Environment;
    use carbide::state::{LocalState, State};
    use crate::scale::{Axis, LinearScale};
    use crate::Scale;

    /// A fixed start, end and step replace the fitted ones exactly, whatever the data: an angle
    /// from 0 to 360 in steps of 45, for data from 10 to 320.
    #[test]
    fn min_max_and_step_replace_the_fitted_range() {
        let mut scale = LinearScale::new(Axis::Radial).min(0.0).max(360.0).step(45.0);
        scale.set_range(10.0, 320.0);

        assert_eq!(scale.range(), 0.0..360.0);
        assert_eq!(scale.ticks(), &[0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0, 360.0]);
        assert_eq!(scale.normalize(180.0), 0.5);
    }

    /// With only one end fixed, the other is fitted and rounded out to a nice step as before; a
    /// fixed end off the steps keeps a tick of its own, the others on whole multiples of the step.
    #[test]
    fn one_fixed_end_keeps_the_other_fitted() {
        let mut from_zero = LinearScale::new(Axis::Vertical).min(0.0);
        from_zero.set_range(5.0, 87.0);
        assert_eq!(from_zero.range(), 0.0..90.0);
        assert_eq!(from_zero.ticks().len(), 10);

        let mut from_three = LinearScale::new(Axis::Vertical).min(3.0);
        from_three.set_range(5.0, 50.0);
        assert_eq!(from_three.range(), 3.0..50.0);
        assert_eq!(&from_three.ticks()[..3], &[3.0, 5.0, 10.0], "the fixed start, then whole steps of 5");

        let mut up_to = LinearScale::new(Axis::Vertical).max(100.0);
        up_to.set_range(500.0, 900.0);
        assert_eq!(up_to.range().end, 100.0, "a fixed end below the data moves the start with it");
        assert!(up_to.range().start <= 100.0);
    }

    /// A step too small for the range, or not above zero, is ignored, and a nice one picked.
    #[test]
    fn unusable_steps_are_ignored() {
        let mut tiny = LinearScale::new(Axis::Vertical).min(0.0).max(1000.0).step(0.01);
        tiny.set_range(0.0, 1000.0);
        assert_eq!(tiny.ticks().len(), 11, "100 000 ticks would be too many: {:?}", tiny.ticks().len());

        let mut negative = LinearScale::new(Axis::Vertical).step(-5.0);
        negative.set_range(0.0, 10.0);
        assert_eq!(negative.ticks(), &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
    }

    /// The fixed ends are states, read as they were last synced, so a scale can be zoomed by
    /// changing them.
    #[test]
    fn fixed_ends_are_states() {
        let min = LocalState::new(0.0);
        let mut scale = LinearScale::new(Axis::Vertical).min(min.clone()).max(10.0);
        let mut env = Environment::new();

        scale.sync(&mut env);
        scale.set_range(2.0, 8.0);
        assert_eq!(scale.range(), 0.0..10.0);

        let mut min = min;
        *min.value_mut() = 4.0;
        assert!(scale.sync(&mut env), "a changed end is a change");
        scale.set_range(2.0, 8.0);
        assert_eq!(scale.range(), 4.0..10.0);
    }

    /// A range where min equals max, as from a dataset of one point or of equal values, is
    /// widened by 5% of the value (or by 1 at zero), and gets ticks around the value. It used to panic: rounding the ticks computed
    /// `10u32.pow(10)`, which overflows, for a range like 0.95..1.05.
    #[test]
    fn equal_min_and_max_get_a_range_around_the_value() {
        for value in [1.0, 0.0, -3.0, 0.1, 1234.5, 1e-7] {
            let mut scale = LinearScale::new(Axis::Vertical);
            scale.set_range(value, value);

            let ticks = scale.ticks();
            assert!(ticks.len() >= 2, "{value}: expected ticks around the value, got {ticks:?}");
            assert!(Scale::min(&scale) < value && value < Scale::max(&scale), "{value}: the value should be inside {}..{}", Scale::min(&scale), Scale::max(&scale));

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

        assert!((Scale::min(&scale) - 1.9).abs() < 1e-12, "the scale should start at 1.9, not {}", Scale::min(&scale));
        assert!((Scale::max(&scale) - 2.1).abs() < 1e-12, "the scale should end at 2.1, not {}", Scale::max(&scale));
        assert!((scale.normalize(2.0) - 0.5).abs() < 1e-9, "the point should be centred");
        assert!((scale.ticks()[0] - 1.9).abs() < 1e-12, "the first tick should be 1.9, not {}", scale.ticks()[0]);
    }

    /// Ticks whose rounding needs ten or more decimals: the factor is a float, so it can't
    /// overflow.
    #[test]
    fn ticks_with_many_decimals_do_not_overflow() {
        let mut scale = LinearScale::new(Axis::Vertical);
        scale.set_range(0.95, 1.05);
        assert!(scale.ticks().len() >= 2);

        let mut scale = LinearScale::new(Axis::Vertical);
        scale.set_range(1e-9, 3e-9);
        assert!(scale.ticks().iter().all(|tick: &Scalar| tick.is_finite()));
    }

    /// `denormalize` undoes `normalize`, with and without an offset, and carries on past the
    /// scale's ends.
    #[test]
    fn denormalize_is_the_inverse_of_normalize() {
        let mut scale = LinearScale::new(Axis::Horizontal);
        scale.set_range(3.0, 97.0);

        for value in [-20.0, 0.0, 37.5, 100.0, 140.0] {
            assert!((scale.denormalize(scale.normalize(value)) - value).abs() < 1e-9, "{value}");
        }
        assert_eq!(scale.denormalize(0.0), 0.0, "the start of the nice range");

        scale.set_offset(true);
        for value in [-20.0, 0.0, 37.5, 100.0] {
            assert!((scale.denormalize(scale.normalize(value)) - value).abs() < 1e-9, "{value} with an offset");
        }
        assert!(scale.denormalize(0.0) < 0.0, "with an offset, the area starts half a step before the range");
    }

    /// A zoomed scale maps its window of the fitted range onto the chart area, and its ticks are
    /// nice ones for the part shown, all inside it; its fitted range stays the same.
    #[test]
    fn zoom_maps_the_window_and_ticks_the_part_shown() {
        let mut scale = LinearScale::new(Axis::Horizontal).zoom(crate::ZoomWindow::new(0.25, 0.5));
        scale.set_range(0.0, 100.0);

        assert_eq!(Scale::range(&scale), 0.0..100.0, "it still fits all the data");
        assert_eq!(scale.normalize(25.0), 0.0);
        assert_eq!(scale.normalize(50.0), 1.0);
        assert_eq!(scale.denormalize(0.5), 37.5);
        assert_eq!(scale.ticks(), &[25.0, 30.0, 35.0, 40.0, 45.0, 50.0]);

        let mut whole = LinearScale::new(Axis::Horizontal).zoom(crate::ZoomWindow::WHOLE);
        whole.set_range(0.0, 100.0);
        assert_eq!(whole.ticks().len(), 11, "the whole range is ticked as without a zoom");
    }

    /// A radial scale isn't zoomed, whatever window it is given.
    #[test]
    fn radial_scales_arent_zoomed() {
        let mut scale = LinearScale::new(Axis::Radial).zoom(crate::ZoomWindow::new(0.25, 0.5));
        scale.set_range(0.0, 100.0);

        assert!(scale.axis_options().window().is_whole());
        assert_eq!(scale.normalize(100.0), 1.0);
    }
}
