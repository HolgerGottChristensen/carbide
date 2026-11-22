//! The chart elements of this crate, each drawn by a chart through the public API alone.

use carbide::chart::{Axis, Chart, DataColor, DatasetExt, LegendPosition, LineController, ScaleId, ScatterController};
use carbide::color::{GREEN, RED};
use carbide::draw::Dimension;
use carbide::testing::Harness;
use chart_extensions::{Kelvin, LogScale, LollipopController};

use support::Framed;

/// A line through readings in `Kelvin`, a coordinate type of this crate's own: the y scale is in
/// degrees Celsius, as `Kelvin` places itself, and the reading at absolute zero is missing, so the
/// line has a gap there.
#[test]
fn kelvin_readings_in_a_line() {
    let readings = vec![(0.0, Kelvin(285.2)), (1.0, Kelvin(287.9)), (2.0, Kelvin(290.1)), (3.0, Kelvin(0.0)), (4.0, Kelvin(288.4)), (5.0, Kelvin(284.6)), (6.0, Kelvin(283.0))];

    let chart = Chart::new(LineController::new(readings))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("kelvin_readings_in_a_line");
}

/// Points spanning five orders of magnitude on a `LogScale` set as the chart's y scale: each power
/// of ten takes the same height, with a tick at each, from 100 to 10 000 000.
#[test]
fn points_on_a_log_scale() {
    let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];

    let chart = Chart::new(ScatterController::new(populations))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical).title("Inhabitants"))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("points_on_a_log_scale");
}

/// Two datasets drawn by the `LollipopController`, a controller of this crate's own, with options
/// set through `DatasetExt`: stems grow from zero, up or down; the first dataset takes the
/// fallback colour, as palette colours aren't public, the second its own green, and one of its
/// points its own red; the missing point has no lollipop; and both named datasets are in the
/// legend.
#[test]
fn lollipops() {
    let first = vec![(0.0, 4.0), (1.0, -2.5), (2.0, 6.0), (3.0, f64::NAN), (4.0, 3.0)];
    let second = vec![(0.5, 2.0), (1.5, 5.0), (2.5, -4.0), (3.5, 1.5)];
    let marked = vec![(4.5, 7.0)].color(DataColor::Color(RED));

    let chart = Chart::new(LollipopController::new((
        first.label("Fallback colour"),
        second.color(DataColor::Color(GREEN)).label("Own colour"),
        marked,
    )))
        .legend(LegendPosition::Bottom)
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("lollipops");
}

/// Lollipops on a `LogScale`, where zero isn't on the scale: the stems start at its bottom end.
#[test]
fn lollipops_on_a_log_scale() {
    let populations = vec![(0.0, 120.0), (1.0, 2_400.0), (2.0, 58_000.0), (3.0, 730_000.0), (4.0, 9_100_000.0)];

    let chart = Chart::new(LollipopController::new(populations))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("lollipops_on_a_log_scale");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide::color::BLUE;
    use carbide::i18n::{locale, LocaleExt};
    use carbide::state::ValueState;
    use carbide::widget::Widget;

    /// How every chart in the snapshots is framed.
    pub trait Framed: Widget + Sized {
        /// Draws a blue border around the chart and formats its tick labels in Danish, so the
        /// snapshots don't depend on the machine's locale.
        fn framed(self) -> impl Widget {
            self.border().color(BLUE)
                .locale(ValueState::new(locale!("da")))
        }
    }

    impl<W: Widget> Framed for W {}
}
