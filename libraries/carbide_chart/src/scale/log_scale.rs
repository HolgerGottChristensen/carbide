use std::ops::Range;

use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::{AnyReadState, IntoReadState, ReadState, ReadStateExtNew};

use crate::scale::{Axis, AxisOptions, AxisPosition, Scale, MIN_ZOOM_SPAN};

/// At most how many labelled ticks a log scale shows: more powers than this show every second,
/// fifth or tenth one.
const MAX_TICKS: usize = 11;

/// The strides between labelled powers, smallest first, when more powers show than fit.
const STRIDES: [i32; 10] = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000];

/// A logarithmic scale: equal ratios are equal distances, so 1, 10, 100 and 1000 are evenly
/// spaced. For data spanning several powers of ten, such as populations or frequencies.
///
/// - **Only values above zero have a place.** Points whose x or y is zero or negative aren't
///   drawn, and a line breaks at them, as at a missing point (see `Scale::place`). The range
///   starts at the smallest positive value (see `DataBounds::smallest_positive`), so bars, which
///   include zero, still fit their values. Bars and filled areas grow from the scale's minimum.
/// - **Ticks fall on the powers of the base,** labelled as numbers in the locale, with the range
///   rounded out to whole powers. When more than 11 powers would show, every second, fifth or
///   tenth power gets a tick instead. When fewer than 10 show and the base is at least 3, minor
///   ticks without labels mark 2 to base − 1 times each power (see `Scale::minor_ticks`).
/// - **The base** (10 by default) only places the ticks: the mapping is the same whatever the
///   base, as a ratio of logarithms doesn't depend on it.
///
/// A dataset's values don't say they are logarithmic, so the scale is set on the chart
/// (`Chart::scale`) for the dataset's scale id.
#[derive(Clone, Debug)]
pub struct LogScale {
    axis: Axis,
    axis_options: AxisOptions,
    offset: bool,
    base: Box<dyn AnyReadState<T=Scalar>>,
    smallest_positive: Option<Scalar>,
    range: Range<Scalar>,
    nice_range: Range<Scalar>,
    ticks: Vec<Scalar>,
    minor_ticks: Vec<Scalar>,
    /// The ticks and minor ticks in the zoom window, when the scale shows only part of its range
    /// (see `zoom`).
    visible_ticks: Option<(Vec<Scalar>, Vec<Scalar>)>,
}

impl LogScale {
    pub fn new(axis: Axis) -> LogScale {
        let mut scale = LogScale {
            axis,
            axis_options: AxisOptions::new(axis),
            offset: false,
            base: 10.0.as_dyn_read(),
            smallest_positive: None,
            range: 1.0..10.0,
            nice_range: 1.0..10.0,
            ticks: vec![],
            minor_ticks: vec![],
            visible_ticks: None,
        };

        scale.recalculate_ticks();
        scale
    }

    /// The base whose powers get the labelled ticks, a value or a state of one. Defaults to 10.
    /// A base of 1 or less, which has no powers to tick, is taken as 10.
    pub fn base(mut self, base: impl IntoReadState<Scalar>) -> LogScale {
        self.base = base.into_read_state().as_dyn_read();
        self
    }

    /// See `LinearScale::position`.
    pub fn position(mut self, position: impl IntoReadState<AxisPosition>) -> LogScale {
        self.axis_options = self.axis_options.with_position(position);
        self
    }

    /// See `LinearScale::weight`.
    pub fn weight(mut self, weight: impl IntoReadState<i32>) -> LogScale {
        self.axis_options = self.axis_options.with_weight(weight);
        self
    }

    /// See `LinearScale::grid_lines`. Minor ticks get grid lines too, fainter ones.
    pub fn grid_lines(mut self, grid_lines: impl IntoReadState<bool>) -> LogScale {
        self.axis_options = self.axis_options.with_grid_lines(grid_lines);
        self
    }

    /// See `LinearScale::tick_marks`.
    pub fn tick_marks(mut self, tick_marks: impl IntoReadState<bool>) -> LogScale {
        self.axis_options = self.axis_options.with_tick_marks(tick_marks);
        self
    }

    /// See `LinearScale::title`.
    pub fn title(mut self, title: impl IntoReadState<String>) -> LogScale {
        self.axis_options = self.axis_options.with_title(title);
        self
    }

    crate::scale::axis_options::polar_options!(LogScale);
    crate::scale::axis_options::zoom_options!(LogScale);

    /// The base as last synced, or 10 when it is 1 or less, or not a number.
    fn base_value(&self) -> Scalar {
        let base = *self.base.value();
        if base > 1.0 && base.is_finite() { base } else { 10.0 }
    }

    /// The mapped range, in natural logarithms: with an offset, widened by half the average step
    /// between ticks at both ends.
    fn log_range(&self) -> (Scalar, Scalar) {
        let (mut start, mut end) = (self.nice_range.start.ln(), self.nice_range.end.ln());

        if self.offset && self.ticks.len() > 1 {
            let half_step = (end - start) / (self.ticks.len() - 1) as Scalar / 2.0;
            start -= half_step;
            end += half_step;
        }

        (start, end)
    }
}

/// How many decimals `value` needs to show its first two significant digits: none from one up,
/// and enough below one that 0.0001 and 0.25 show in full. Trailing zeros are left off.
fn decimals(value: Scalar) -> usize {
    if value >= 1.0 || value <= 0.0 { 0 } else { ((-value.log10()).ceil() as usize + 1).min(20) }
}

#[cfg(feature = "carbide_fluent")]
fn format_power(value: Scalar, env: &mut Environment) -> String {
    use carbide::state::{StateSync, ValueState};
    use carbide_fluent::LocalizedNumber;

    let mut number = LocalizedNumber::new(ValueState::new(value)).maximum_fraction_digits(ValueState::new(Some(decimals(value))));
    number.sync(env);
    number.value().to_string()
}

#[cfg(not(feature = "carbide_fluent"))]
fn format_power(value: Scalar, _env: &mut Environment) -> String {
    let text = format!("{:.*}", decimals(value), value);
    if text.contains('.') { text.trim_end_matches('0').trim_end_matches('.').to_string() } else { text }
}

/// `exponent` rounded to a whole number when it is within floating-point noise of one, so the
/// logarithm of a power, such as `log10(1000) = 2.9999999999999996`, counts as that power.
fn snap(exponent: Scalar) -> Scalar {
    if (exponent - exponent.round()).abs() < 1e-9 { exponent.round() } else { exponent }
}

impl Scale for LogScale {
    fn axis(&self) -> Axis {
        self.axis
    }

    fn axis_options(&self) -> &AxisOptions {
        &self.axis_options
    }

    fn set_polar_role(&mut self, role: Option<crate::scale::PolarRole>) {
        self.axis_options.set_polar_role(role);
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        let base = self.base.sync(env);
        self.axis_options.sync(env) | base
    }

    /// The fraction of the way from the start of the range, in logarithms. A value at or below
    /// zero, which has no place but can reach here as a mark's value (an error bar's low end, say),
    /// is taken as the scale's minimum, so it is drawn at the edge rather than at no pixel at all.
    fn normalize(&self, value: Scalar) -> Scalar {
        let (start, end) = self.log_range();
        let log = if value > 0.0 { value.ln() } else { self.nice_range.start.ln() };

        self.axis_options.window().zoom((log - start) / (end - start))
    }

    fn denormalize(&self, normalized: Scalar) -> Scalar {
        let (start, end) = self.log_range();
        (start + self.axis_options.window().unzoom(normalized) * (end - start)).exp()
    }

    fn set_offset(&mut self, offset: bool) {
        self.offset = offset;
    }

    /// Values above zero are placed at themselves; zero and below have no place.
    fn place(&self, value: Scalar) -> Option<Scalar> {
        (value > 0.0 && value.is_finite()).then_some(value)
    }

    /// The number, formatted for the locale with as many decimals as a power below one needs, so
    /// 0.0001 doesn't round to 0 as it would with a linear scale's few decimals.
    fn label(&self, value: Scalar, env: &mut Environment) -> Option<String> {
        Some(format_power(value, env))
    }

    fn set_smallest_positive(&mut self, smallest: Option<Scalar>) {
        self.smallest_positive = smallest;
    }

    fn minor_ticks(&self) -> &[Scalar] {
        self.visible_ticks.as_ref().map_or(&self.minor_ticks, |(_, minor)| minor)
    }

    fn min(&self) -> Scalar {
        self.nice_range.start
    }

    fn max(&self) -> Scalar {
        self.nice_range.end
    }

    /// Fits the scale to `min` and `max`, starting from the smallest positive value given by
    /// `set_smallest_positive` when `min` is zero or below. Without any value above zero, the
    /// scale spans 1 to the base; a single value is widened to a power either side.
    fn set_range(&mut self, min: Scalar, max: Scalar) {
        let base = self.base_value();
        let start = if min > 0.0 && min.is_finite() { Some(min) } else { self.smallest_positive };

        self.range = match start {
            Some(start) if max.is_finite() && max >= start => start..max,
            _ => 1.0..base,
        };

        if self.range.start == self.range.end {
            self.range = self.range.start / base..self.range.end * base;
        }

        self.recalculate_ticks();
    }

    fn range(&self) -> Range<Scalar> {
        self.nice_range.clone()
    }

    /// Rounds the range out to whole powers of the base, ticks every power, or every second,
    /// fifth or tenth when more than `MAX_TICKS` would show, and adds the minor ticks.
    fn recalculate_ticks(&mut self) {
        let base = self.base_value();
        let exponent = |value: Scalar| snap(value.ln() / base.ln());

        let mut low = exponent(self.range.start).floor() as i32;
        let mut high = exponent(self.range.end).ceil() as i32;
        if low == high {
            high += 1;
        }

        // The powers rounded out to multiples of a stride, and how many ticks they give.
        let rounded = |stride: i32| (low.div_euclid(stride) * stride, -((-high).div_euclid(stride)) * stride);
        let tick_count = |stride: i32| {
            let (low, high) = rounded(stride);
            ((high - low) / stride) as usize + 1
        };

        let stride = STRIDES.into_iter().find(|stride| tick_count(*stride) <= MAX_TICKS).unwrap_or(STRIDES[STRIDES.len() - 1]);
        (low, high) = rounded(stride);

        self.ticks = (low..=high).step_by(stride as usize).map(|power| base.powi(power)).collect();
        self.nice_range = base.powi(low)..base.powi(high);

        let powers = (high - low) as usize;
        self.minor_ticks = if stride == 1 && powers < 10 && base >= 3.0 {
            (low..high)
                .flat_map(|power| {
                    let at = base.powi(power);
                    (2..).map(move |multiple| multiple as Scalar).take_while(move |multiple| *multiple < base).map(move |multiple| multiple * at)
                })
                .collect()
        } else {
            vec![]
        };

        self.zoom_ticks();
    }

    fn ticks(&self) -> &[Scalar] {
        self.visible_ticks.as_ref().map_or(&self.ticks, |(ticks, _)| ticks)
    }
}

impl LogScale {
    /// Decides the window the scale shows from the zoom asked for, and, when it shows only part
    /// of the range, the ticks inside it: the powers and their multiples there, with the
    /// multiples labelled when fewer than two powers are left to label.
    fn zoom_ticks(&mut self) {
        let window = self.axis_options.zoom().limited(MIN_ZOOM_SPAN);
        self.axis_options.set_window(window);
        self.visible_ticks = None;

        if window.is_whole() {
            return;
        }

        let base = self.base_value();
        let (low, high) = (self.denormalize(0.0), self.denormalize(1.0));
        let inside = |value: &Scalar| *value >= low * (1.0 - 1e-9) && *value <= high * (1.0 + 1e-9);

        let first = (low.ln() / base.ln()).floor() as i32;
        let last = (high.ln() / base.ln()).ceil() as i32;
        let ticks: Vec<Scalar> = (first..=last).map(|power| base.powi(power)).filter(inside).collect();
        let minor: Vec<Scalar> = if base >= 3.0 && last - first < 10 {
            (first..last)
                .flat_map(|power| {
                    let at = base.powi(power);
                    (2..).map(move |multiple| multiple as Scalar).take_while(move |multiple| *multiple < base).map(move |multiple| multiple * at)
                })
                .filter(inside)
                .collect()
        } else {
            vec![]
        };

        self.visible_ticks = Some(if ticks.len() >= 2 {
            (ticks, minor)
        } else {
            let mut all: Vec<Scalar> = ticks.into_iter().chain(minor).collect();
            all.sort_by(|a, b| a.total_cmp(b));
            (all, vec![])
        });
    }
}

#[cfg(test)]
mod tests {
    use carbide::draw::Scalar;

    use crate::scale::{Axis, Scale};
    use super::LogScale;

    fn fitted(scale: LogScale, min: Scalar, max: Scalar) -> LogScale {
        let mut scale = scale;
        scale.set_range(min, max);
        scale
    }

    /// Equal ratios are equal distances, whatever the base: 1, 10, 100 and 1000 are evenly
    /// spaced on base 10 and on base 2 alike.
    #[test]
    fn powers_are_evenly_spaced_whatever_the_base() {
        for scale in [LogScale::new(Axis::Vertical), LogScale::new(Axis::Vertical).base(2.0)] {
            let scale = fitted(scale, 1.0, 1000.0);
            let steps: Vec<Scalar> = [1.0, 10.0, 100.0].iter().map(|value| scale.normalize(value * 10.0) - scale.normalize(*value)).collect();
            assert!(steps.windows(2).all(|pair| (pair[0] - pair[1]).abs() < 1e-12), "{steps:?}");
        }
    }

    /// The range is rounded out to whole powers, with a tick on each and minor ticks at 2 to 9
    /// times each power between them.
    #[test]
    fn ticks_fall_on_powers_with_minor_ticks_between() {
        let scale = fitted(LogScale::new(Axis::Vertical), 3.0, 450.0);

        assert_eq!(scale.ticks(), &[1.0, 10.0, 100.0, 1000.0]);
        assert_eq!((scale.min(), scale.max()), (1.0, 1000.0));
        assert_eq!(scale.minor_ticks().len(), 3 * 8, "2 to 9 times each of the three powers below the top");
        assert_eq!(&scale.minor_ticks()[..3], &[2.0, 3.0, 4.0]);

        let exact = fitted(LogScale::new(Axis::Vertical), 1.0, 1000.0);
        assert_eq!(exact.ticks(), &[1.0, 10.0, 100.0, 1000.0], "a power already at the end isn't rounded past, despite log10(1000) not being exactly 3");
    }

    /// Too many powers for every one to get a tick: every second, fifth or tenth does, the
    /// smallest stride that fits, and there are no minor ticks.
    #[test]
    fn many_powers_get_a_stride() {
        let scale = fitted(LogScale::new(Axis::Vertical), 1e-3, 1e15);

        assert_eq!(scale.ticks().len(), 11, "every second power, from 1e-4 to 1e16");
        assert_eq!((scale.ticks()[0], scale.ticks()[10]), (1e-4, 1e16));
        assert!(scale.minor_ticks().is_empty());
    }

    /// Base 2 ticks its powers, without minor ticks, as there is no whole multiple between them.
    #[test]
    fn base_two() {
        let scale = fitted(LogScale::new(Axis::Horizontal).base(2.0), 3.0, 50.0);

        assert_eq!(scale.ticks(), &[2.0, 4.0, 8.0, 16.0, 32.0, 64.0]);
        assert!(scale.minor_ticks().is_empty());
    }

    /// Labels show powers below one in full, which a linear scale's few decimals would round to 0,
    /// and powers from one up without decimals.
    #[test]
    fn small_powers_are_labelled_in_full() {
        use carbide::environment::Environment;

        let scale = fitted(LogScale::new(Axis::Vertical), 1e-4, 1e4);
        let mut env = Environment::new();
        let labels: Vec<String> = [1e-4, 0.25, 100.0].iter().map(|value| scale.label(*value, &mut env).unwrap()).collect();

        assert!(labels[0].ends_with("0001") && labels[0].starts_with('0'), "{}", labels[0]);
        assert!(labels[1].ends_with("25"), "{}", labels[1]);
        assert_eq!(labels[2], "100");
    }

    /// Zero and below have no place. The range starts at the smallest positive value when the
    /// data reach down to zero, as bars do, and without positive data the scale spans 1 to the
    /// base. A value below zero reaching `normalize` is drawn at the scale's minimum.
    #[test]
    fn zero_and_below_have_no_place() {
        let mut scale = LogScale::new(Axis::Vertical);
        assert_eq!(scale.place(0.0), None);
        assert_eq!(scale.place(-4.0), None);
        assert_eq!(scale.place(5.0), Some(5.0));

        scale.set_smallest_positive(Some(12.0));
        scale.set_range(0.0, 5000.0);
        assert_eq!((scale.min(), scale.max()), (10.0, 10_000.0), "from the smallest positive value, not from zero");
        assert_eq!(scale.normalize(-1.0), scale.normalize(scale.min()));

        scale.set_smallest_positive(None);
        scale.set_range(-5.0, 0.0);
        assert_eq!((scale.min(), scale.max()), (1.0, 10.0), "no positive data");

        scale.set_smallest_positive(Some(50.0));
        scale.set_range(50.0, 50.0);
        assert_eq!((scale.min(), scale.max()), (1.0, 1000.0), "a single value is widened a power either side");
    }

    /// `denormalize` undoes `normalize` along the logarithm, with and without an offset.
    #[test]
    fn denormalize_is_the_inverse_of_normalize() {
        let mut scale = fitted(LogScale::new(Axis::Vertical), 3.0, 4000.0);

        for value in [1.0, 3.0, 42.0, 10_000.0] {
            assert!((scale.denormalize(scale.normalize(value)) / value - 1.0).abs() < 1e-9, "{value}");
        }
        assert!((scale.denormalize(0.5) - 100.0).abs() < 1e-9, "halfway between 1 and 10 000 is 100");

        scale.set_offset(true);
        for value in [1.0, 42.0, 10_000.0] {
            assert!((scale.denormalize(scale.normalize(value)) / value - 1.0).abs() < 1e-9, "{value} with an offset");
        }
    }

    /// A zoomed log scale shows the powers inside its window, or, with fewer than two of them, the
    /// multiples of the powers there, labelled.
    #[test]
    fn zoom_ticks_the_powers_or_multiples_shown() {
        let mut scale = fitted(LogScale::new(Axis::Vertical).zoom(crate::ZoomWindow::new(0.0, 0.5)), 1.0, 10_000.0);
        assert_eq!(scale.ticks(), &[1.0, 10.0, 100.0]);

        // From 100 to 631: one power, so its multiples are labelled.
        scale = fitted(LogScale::new(Axis::Vertical).zoom(crate::ZoomWindow::new(0.5, 0.7)), 1.0, 10_000.0);
        assert_eq!(scale.ticks(), &[100.0, 200.0, 300.0, 400.0, 500.0, 600.0]);
        assert!(scale.minor_ticks().is_empty());
    }
}
