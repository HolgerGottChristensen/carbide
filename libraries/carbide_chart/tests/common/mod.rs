//! Helpers shared by the rendering tests in this folder. Each test file is its own crate and
//! declares `mod common;`, so not every file uses every helper.
#![allow(dead_code)]

use carbide_chart::{BarController, BarDataSet, Chart, IntoSequence, Sequence};
use carbide_core::color::BLUE;
use carbide_core::state::ValueState;
use carbide_core::widget::Widget;
use carbide_fluent::{locale, LocaleExt};

/// How every chart in the snapshots is framed.
pub trait Framed: Widget + Sized {
    /// Draws a blue border around the chart and formats its tick labels in Danish.
    fn framed(self) -> impl Widget {
        // `.border()` alone picks a random colour on every run, which makes snapshots flaky.
        self.border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }
}

impl<W: Widget> Framed for W {}

/// A chart with only a `BarController` for `datasets` on the default scales: no lines or
/// points on top.
pub fn bar_chart<I, M, D>(datasets: I) -> impl Widget
where
    I: IntoSequence<dyn BarDataSet, M, Sequence=D>,
    D: Sequence<dyn BarDataSet>,
{
    Chart::new(BarController::new(datasets))
        .framed()
}
