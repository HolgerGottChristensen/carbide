use std::ops::Range;

use carbide::chart::{Axis, AxisOptions, AxisPosition, LabelRotation, Scale, ZoomWindow};
use carbide::draw::{Alignment, Scalar};
use carbide::environment::Environment;
use carbide::state::{IntoReadState, IntoState};

/// A scale that places values by their base 10 logarithm, so each power of ten takes the same
/// length along the axis. The range is rounded out to whole powers of ten, with a tick at each.
///
/// Only positive values have a logarithm: a range starting at or below zero starts at a thousandth
/// of its end instead, and points at or below zero aren't drawn: a line has a gap there.
///
/// It zooms when given a zoom window (`zoom`), as the built-in scales do: values are mapped
/// through the window, and the ticks are the powers of ten inside it, or every multiple of a power
/// inside it when fewer than two powers are.
#[derive(Clone, Debug)]
pub struct LogScale {
    axis: Axis,
    axis_options: AxisOptions,
    /// The exponents of the powers of ten the range was rounded out to.
    exponents: Range<i32>,
    ticks: Vec<Scalar>,
}

impl LogScale {
    pub fn new(axis: Axis) -> LogScale {
        let mut scale = LogScale {
            axis,
            axis_options: AxisOptions::new(axis),
            exponents: 0..1,
            ticks: vec![],
        };
        scale.recalculate_ticks();
        scale
    }

    /// Draws the scale on the given side of the chart area, a side or a state of one.
    pub fn position(mut self, position: impl IntoReadState<AxisPosition>) -> LogScale {
        self.axis_options = self.axis_options.with_position(position);
        self
    }

    /// Gives the scale's axis a title, a text or a state of one.
    pub fn title(mut self, title: impl IntoReadState<String>) -> LogScale {
        self.axis_options = self.axis_options.with_title(title);
        self
    }

    /// Shows only part of the fitted range, a `ZoomWindow` or a state of one, which the chart's
    /// gestures pan and zoom.
    pub fn zoom(mut self, window: impl IntoState<ZoomWindow>) -> LogScale {
        self.axis_options = self.axis_options.with_zoom(window);
        self
    }

    /// Fits the scale to the data inside the zoom windows of the scales its data is placed along,
    /// a value or a state of one: a value scale whose index scale is zoomed fits the values shown.
    pub fn fit_visible(mut self, fit_visible: impl IntoReadState<bool>) -> LogScale {
        self.axis_options = self.axis_options.with_fit_visible(fit_visible);
        self
    }

    /// Draws grid lines at the ticks, or not with `false`, a value or a state of one.
    pub fn grid_lines(mut self, grid_lines: impl IntoReadState<bool>) -> LogScale {
        self.axis_options = self.axis_options.with_grid_lines(grid_lines);
        self
    }

    /// How far the tick labels are turned: a `LabelRotation`, an `Angle` or a number of degrees,
    /// or a state of any of them. The chart turns, aligns and thins out the labels of any scale
    /// from its axis options, so a scale of its own only passes the option on.
    pub fn label_rotation(mut self, rotation: impl IntoReadState<LabelRotation>) -> LogScale {
        self.axis_options = self.axis_options.with_label_rotation(rotation);
        self
    }

    /// The point of each tick label at the end of its tick mark, an `Alignment` or a state of an
    /// `Option<Alignment>`, where `None` picks it automatically.
    pub fn label_alignment(mut self, alignment: impl IntoReadState<Option<Alignment>>) -> LogScale {
        self.axis_options = self.axis_options.with_label_alignment(alignment);
        self
    }

    /// Whether tick labels too close together are thinned out, a value or a state of one.
    pub fn label_skip(mut self, skip: impl IntoReadState<bool>) -> LogScale {
        self.axis_options = self.axis_options.with_label_skip(skip);
        self
    }

    /// The share of the fitted range `value` is at, in logarithms, before the zoom window.
    fn share(&self, value: Scalar) -> Scalar {
        let start = self.exponents.start as Scalar;
        let end = self.exponents.end as Scalar;
        (value.log10() - start) / (end - start)
    }

    /// The value at `share` of the fitted range: the inverse of `share`.
    fn at_share(&self, share: Scalar) -> Scalar {
        let start = self.exponents.start as Scalar;
        let end = self.exponents.end as Scalar;
        10f64.powf(start + share * (end - start))
    }
}

impl Scale for LogScale {
    fn axis(&self) -> Axis {
        self.axis
    }

    fn axis_options(&self) -> &AxisOptions {
        &self.axis_options
    }

    /// The options, so the chart's gestures can set the zoom (see `zoom`).
    fn axis_options_mut(&mut self) -> Option<&mut AxisOptions> {
        Some(&mut self.axis_options)
    }

    /// Syncs the title and the zoom. A changed zoom refits the chart, which calls `set_range`, so
    /// the window and the ticks follow it.
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.axis_options.sync(env)
    }

    /// Where `value` falls in the zoom window, by its logarithm.
    fn normalize(&self, value: Scalar) -> Scalar {
        self.axis_options.window().zoom(self.share(value))
    }

    /// The inverse of `normalize`, so a position in the chart area, such as the pointer's, reads
    /// back as the value drawn there.
    fn denormalize(&self, normalized: Scalar) -> Scalar {
        self.at_share(self.axis_options.window().unzoom(normalized))
    }

    /// Values above zero are placed at themselves; zero and below have no logarithm, so they have
    /// no place, and points at them aren't drawn.
    fn place(&self, value: Scalar) -> Option<Scalar> {
        (value > 0.0 && value.is_finite()).then_some(value)
    }

    fn min(&self) -> Scalar {
        10f64.powi(self.exponents.start)
    }

    fn max(&self) -> Scalar {
        10f64.powi(self.exponents.end)
    }

    fn set_range(&mut self, min: Scalar, max: Scalar) {
        let max = if max.is_finite() && max > 0.0 { max } else { 1.0 };
        let min = if min.is_finite() && min > 0.0 { min.min(max) } else { max / 1000.0 };

        let start = min.log10().floor() as i32;
        // A range within one power of ten still spans one, so it has two ticks.
        let end = (max.log10().ceil() as i32).max(start + 1);

        self.exponents = start..end;
        self.recalculate_ticks();
    }

    /// Decides the window shown, the zoom asked for kept within the range and at least the
    /// smallest span wide, then ticks the powers of ten inside it, or, when fewer than two powers
    /// are, every multiple of a power inside it.
    fn recalculate_ticks(&mut self) {
        let window = self.axis_options.zoom().limited(self.axis_options.min_span());
        self.axis_options.set_window(window);

        let (low, high) = (self.denormalize(0.0), self.denormalize(1.0));
        // A little slack, so a power on the window's edge counts as inside it.
        let inside = |value: &Scalar| *value >= low * (1.0 - 1e-9) && *value <= high * (1.0 + 1e-9);
        let (first, last) = (low.log10().floor() as i32, high.log10().ceil() as i32);

        let powers: Vec<Scalar> = (first..=last).map(|exponent| 10f64.powi(exponent)).filter(inside).collect();
        self.ticks = if powers.len() >= 2 {
            powers
        } else {
            (first..last)
                .flat_map(|exponent| (1..10).map(move |multiple| multiple as Scalar * 10f64.powi(exponent)))
                .filter(inside)
                .collect()
        };
    }

    fn ticks(&self) -> &[Scalar] {
        &self.ticks
    }

    fn range(&self) -> Range<Scalar> {
        self.min()..self.max()
    }
}
