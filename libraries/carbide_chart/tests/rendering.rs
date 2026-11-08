use carbide_chart::Stepped;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

use support::{example_chart, stacked_horizontal_scales_chart, stacked_scales_chart, two_y_scales_chart};

/// Renders the chart from `carbide/examples/chart/chart.rs` (two datasets, the first one's
/// `stepped` mode driven by a state, Danish locale, bordered) and snapshots it once per `Stepped`
/// variant, the same sequence the example's picker steps through. The datasets are fixed, so
/// each snapshot pins down the linear scale's ticks and grid, the tick labels, the dataset
/// colors and the line path for one stepped mode.
#[test]
fn scatter_chart_stepped_modes() {
    let stepped = LocalState::new(Stepped::None);
    let mut harness = Harness::new(example_chart(stepped.clone()), Dimension::new(400.0, 300.0));

    for (mode, name) in [
        (Stepped::None, "stepped_none"),
        (Stepped::Before, "stepped_before"),
        (Stepped::After, "stepped_after"),
        (Stepped::Middle, "stepped_middle"),
        (Stepped::MiddleVertical, "stepped_middle_vertical"),
    ] {
        stepped.clone().set_value(mode);
        harness.snapshot(name);
    }
}

/// Two controllers on one chart: revenue on the default Y scale (left), and a margin line bound
/// to a custom `percent` scale on the right. Pins down that each dataset is drawn on its own
/// scale, that the right-hand scale gets its ticks, labels and border on the right, and that the
/// chart area makes room for it.
#[test]
fn two_y_scales() {
    let mut harness = Harness::new(two_y_scales_chart(), Dimension::new(400.0, 300.0));

    harness.snapshot("two_y_scales");
}

/// Two scales on the right, as in Chart.js: the first added sits against the chart area, the
/// second in its own band outside it. Pins down that neither scale's ticks, labels or border
/// overlap the other's, and that the chart area gives up both bands.
#[test]
fn stacked_scales_on_one_side() {
    let mut harness = Harness::new(stacked_scales_chart(), Dimension::new(400.0, 300.0));

    harness.snapshot("stacked_scales_on_one_side");
}

/// The horizontal counterpart of `stacked_scales_on_one_side`: a second x scale stacked below the
/// default one, with spacing between their bands. Pins down that horizontal bands stack
/// downwards, each with its own tick marks, labels and border, and that the chart area gives up
/// the height of both bands.
#[test]
fn stacked_horizontal_scales() {
    let mut harness = Harness::new(stacked_horizontal_scales_chart(), Dimension::new(400.0, 300.0));

    harness.snapshot("stacked_horizontal_scales");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Axis, AxisPosition, Chart, DataColor, DatasetExt, LinearScale, ScaleId, ScatterController, Stepped};
    use carbide_core::color::{GREEN, RED};
    use carbide_core::state::{LocalState, ValueState};
    use carbide_core::widget::{Widget, WidgetExt};
    use carbide_fluent::{locale, LocaleExt};

    const PERCENT: ScaleId = ScaleId::new("percent");
    const UNITS: ScaleId = ScaleId::new("units");
    const FAHRENHEIT: ScaleId = ScaleId::new("fahrenheit");

    /// Two series of temperature readings, one measured in °C on the default x scale and one in
    /// °F on a second x scale, which is also at the bottom and so stacks below the default one.
    pub fn stacked_horizontal_scales_chart() -> impl Widget {
        let celsius = vec![(0.0, 2.0), (20.0, 5.0), (40.0, 4.0), (60.0, 8.0), (80.0, 7.0), (100.0, 9.0)];
        let fahrenheit = vec![(32.0, 3.0), (68.0, 6.0), (104.0, 5.5), (140.0, 7.5), (176.0, 9.5), (212.0, 8.0)]
            .x_scale(FAHRENHEIT)
            .color(DataColor::Color(GREEN));

        Chart::new((
            ScatterController::new(celsius),
            ScatterController::new(fahrenheit),
        ))
            .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal))
            .border()
            .locale(ValueState::new(locale!("da")))
    }

    /// `two_y_scales_chart` plus units sold on a third scale, also on the right, added after
    /// `percent` so it stacks outside it.
    pub fn stacked_scales_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let margin = vec![0.12, 0.18, 0.15, 0.08, 0.22, 0.25, 0.31]
            .y_scale(PERCENT)
            .color(DataColor::Color(GREEN));
        let units = vec![1800.0, 2100.0, 2600.0, 1500.0, 3000.0, 3400.0, 3900.0]
            .y_scale(UNITS)
            .color(DataColor::Color(RED));

        Chart::new((
            ScatterController::new(revenue),
            ScatterController::new(margin),
            ScatterController::new(units),
        ))
            .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
            .scale(UNITS, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
            .border()
            .locale(ValueState::new(locale!("da")))
    }

    /// Revenue on the default scales and a margin (as a fraction) on a `percent` scale on the
    /// right, in two controllers.
    pub fn two_y_scales_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let margin = vec![0.12, 0.18, 0.15, 0.08, 0.22, 0.25, 0.31]
            .y_scale(PERCENT)
            .color(DataColor::Color(GREEN));

        Chart::new((
            ScatterController::new(revenue),
            ScatterController::new(margin),
        ))
            .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
            .border()
            .locale(ValueState::new(locale!("da")))
    }

    /// The chart from `carbide/examples/chart/chart.rs`, with the example's `data7` and `data8`.
    pub fn example_chart(stepped: LocalState<Stepped>) -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0].stepped(stepped);
        let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        Chart::new(ScatterController::new((data7, data8)))
            .border()
            .locale(ValueState::new(locale!("da")))
    }
}
