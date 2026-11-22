use std::ops::Range;

use carbide::chart::{Axis, AxisOptions, AxisPosition, Scale};
use carbide::draw::Scalar;
use carbide::environment::Environment;
use carbide::state::IntoReadState;

/// A scale that places values by their base 10 logarithm, so each power of ten takes the same
/// length along the axis. The range is rounded out to whole powers of ten, with a tick at each.
///
/// Only positive values have a logarithm: a range starting at or below zero starts at a thousandth
/// of its end instead, and values at or below zero fall outside the chart area.
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
}

impl Scale for LogScale {
    fn axis(&self) -> Axis {
        self.axis
    }

    fn axis_options(&self) -> &AxisOptions {
        &self.axis_options
    }

    fn sync(&mut self, env: &mut Environment) -> bool {
        self.axis_options.sync(env)
    }

    fn normalize(&self, value: Scalar) -> Scalar {
        let start = self.exponents.start as Scalar;
        let end = self.exponents.end as Scalar;
        (value.log10() - start) / (end - start)
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

    fn recalculate_ticks(&mut self) {
        self.ticks = (self.exponents.start..=self.exponents.end)
            .map(|exponent| 10f64.powi(exponent))
            .collect();
    }

    fn ticks(&self) -> &[Scalar] {
        &self.ticks
    }

    fn range(&self) -> Range<Scalar> {
        self.min()..self.max()
    }
}
