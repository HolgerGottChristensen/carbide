use carbide_chart::Stepped;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

use support::{
    bar_chart, bars_and_line_chart, example_chart, stacked_horizontal_scales_chart, stacked_scales_chart,
    stacked_scales_on_left_chart, stacked_scales_on_top_chart, two_y_scales_chart, x_scale_on_top_chart,
};

/// Renders the chart from `carbide/examples/chart/chart.rs` (two datasets, the first one's
/// `stepped` mode driven by a state, Danish locale, bordered) and snapshots it once per `Stepped`
/// variant, the same sequence the example's picker steps through. The datasets are fixed, so
/// each snapshot pins down the linear scale's ticks and grid, the tick labels, the dataset
/// colors and the line path for one stepped mode.
#[test]
fn scatter_chart_stepped_modes() {
    let stepped = LocalState::new(Stepped::None);
    let mut harness = Harness::new(example_chart(stepped.clone()), Dimension::new(600.0, 500.0));

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
    let mut harness = Harness::new(two_y_scales_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("two_y_scales");
}

/// Two scales on the right, as in Chart.js: the first added sits against the chart area, the
/// second in its own band outside it. Pins down that neither scale's ticks, labels or border
/// overlap the other's, and that the chart area gives up both bands.
#[test]
fn stacked_scales_on_one_side() {
    let mut harness = Harness::new(stacked_scales_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_one_side");
}

/// The horizontal counterpart of `stacked_scales_on_one_side`: a second x scale stacked below the
/// default one, with spacing between their bands. Pins down that horizontal bands stack
/// downwards, each with its own tick marks, labels and border, and that the chart area gives up
/// the height of both bands.
#[test]
fn stacked_horizontal_scales() {
    let mut harness = Harness::new(stacked_horizontal_scales_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_horizontal_scales");
}

/// The example chart with its default x scale moved to the top. Pins down a horizontal scale's
/// top side: tick marks reaching up from the chart area, labels above them, its border along the
/// top edge, its band taken off the top of the chart area, and no band left at the bottom.
#[test]
fn x_scale_on_top() {
    let mut harness = Harness::new(x_scale_on_top_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("x_scale_on_top");
}

/// The temperature chart from `stacked_horizontal_scales`, with both x scales at the top: the
/// default x scale against the chart area, the °F scale stacked above it with spacing between
/// their bands.
#[test]
fn stacked_scales_on_top() {
    let mut harness = Harness::new(stacked_scales_on_top_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_top");
}

/// Two vertical scales on the left: revenue on the default y scale against the chart area, units
/// sold on a second scale stacked outside it with spacing between their bands.
#[test]
fn stacked_scales_on_left() {
    let mut harness = Harness::new(stacked_scales_on_left_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_left");
}

/// Bars and a line in one chart, on the same scales: revenue and costs as grouped bars from a
/// `BarController`, profit as a line from a `ScatterController` drawn over them. Pins down that
/// the two bar datasets sit side by side around each x value, that the first and last groups
/// aren't cut off at the chart area's edges (the x scale's offset), that bars grow from zero,
/// and that the line's points sit at the centre of each group, including the negative month.
#[test]
fn bars_and_line_on_shared_scales() {
    let mut harness = Harness::new(bars_and_line_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bars_and_line_on_shared_scales");
}

/// A plain bar chart: one dataset of positive values, nothing else on the chart (the data from
/// Chart.js's sample bar chart). Pins down the single-dataset bar width, the x scale's offset at
/// both ends, and bars growing up from zero at the bottom of the y scale.
#[test]
fn bar_chart_single_dataset() {
    let mut harness = Harness::new(bar_chart(vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_single_dataset");
}

/// One dataset with both positive and negative values. Pins down that every bar starts at zero,
/// in the middle of the y scale: positive bars grow up from it, negative ones hang down from it.
#[test]
fn bar_chart_positive_and_negative() {
    let mut harness = Harness::new(bar_chart(vec![-12.0, 19.0, -3.0, 5.0, -2.0, 8.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_positive_and_negative");
}

/// Three grouped datasets with both positive and negative values. Pins down that each group
/// splits its slot three ways in dataset order (white, green, red), and that bars on either side
/// of zero in the same group share the zero line.
#[test]
fn grouped_bars_positive_and_negative() {
    let datasets = (
        vec![65.0, -59.0, 80.0, -81.0, 56.0],
        vec![-28.0, 48.0, -40.0, 19.0, 86.0],
        vec![35.0, -10.0, 22.0, 60.0, -45.0],
    );
    let mut harness = Harness::new(bar_chart(datasets), Dimension::new(600.0, 500.0));

    harness.snapshot("grouped_bars_positive_and_negative");
}

/// One dataset of only negative values. Zero is still included in the y scale, so it is the top
/// of the scale and every bar hangs down from it.
#[test]
fn bar_chart_all_negative() {
    let mut harness = Harness::new(bar_chart(vec![-5.0, -12.0, -8.0, -20.0, -3.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_all_negative");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Axis, AxisPosition, BarController, Chart, DataColor, DataSetSequence, DatasetExt, LinearScale, ScaleId, ScatterController, Stepped};
    // `.border()` alone picks a random colour on every run, which makes snapshots flaky.
    use carbide_core::color::{BLUE, GREEN, RED};
    use carbide_core::draw::Scalar;
    use carbide_core::state::{LocalState, ValueState};
    use carbide_core::widget::{Widget, WidgetExt};
    use carbide_fluent::{locale, LocaleExt};

    const PERCENT: ScaleId = ScaleId::new("percent");
    const UNITS: ScaleId = ScaleId::new("units");
    const FAHRENHEIT: ScaleId = ScaleId::new("fahrenheit");

    /// The example's two datasets, with the default x scale replaced by one at the top.
    pub fn x_scale_on_top_chart() -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0];
        let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        Chart::new(ScatterController::new((data7, data8)))
            .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// `stacked_horizontal_scales_chart` with both x scales at the top. X is added first (it is a
    /// default scale, replaced in place), so it stays against the chart area.
    pub fn stacked_scales_on_top_chart() -> impl Widget {
        let celsius = vec![(0.0, 2.0), (20.0, 5.0), (40.0, 4.0), (60.0, 8.0), (80.0, 7.0), (100.0, 9.0)];
        let fahrenheit = vec![(32.0, 3.0), (68.0, 6.0), (104.0, 5.5), (140.0, 7.5), (176.0, 9.5), (212.0, 8.0)]
            .x_scale(FAHRENHEIT)
            .color(DataColor::Color(GREEN));

        Chart::new((
            ScatterController::new(celsius),
            ScatterController::new(fahrenheit),
        ))
            .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top))
            .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Revenue on the default y scale and units sold on a second scale, also on the left, added
    /// after Y so it stacks outside it.
    pub fn stacked_scales_on_left_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let units = vec![1800.0, 2100.0, 2600.0, 1500.0, 3000.0, 3400.0, 3900.0]
            .y_scale(UNITS)
            .color(DataColor::Color(RED));

        Chart::new((
            ScatterController::new(revenue),
            ScatterController::new(units),
        ))
            .scale(UNITS, LinearScale::new(Axis::Vertical).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// A chart with only a `BarController` for `datasets` on the default scales: no lines or
    /// points on top.
    pub fn bar_chart<D>(datasets: D) -> impl Widget
    where
        D: DataSetSequence<X=Scalar, Y=Scalar, Z=Scalar> + Clone,
    {
        Chart::new(BarController::new(datasets))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Revenue and costs as grouped bars, and profit (revenue minus costs, negative in one month)
    /// as a red line over them, all on the default scales.
    pub fn bars_and_line_chart() -> impl Widget {
        let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
        let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0, 150.0, 155.0];
        let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0, 25.0, 35.0]
            .color(DataColor::Color(RED));

        Chart::new((
            BarController::new((revenue, costs)),
            ScatterController::new(profit),
        ))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// Two series of temperature readings, one measured in °C on the default x scale and one in
    /// °F on a second x scale, which is also at the bottom and so stacks below the default one.
    /// The °F scale draws no grid lines, so only the °C scale's lines cross the chart area.
    pub fn stacked_horizontal_scales_chart() -> impl Widget {
        let celsius = vec![(0.0, 2.0), (20.0, 5.0), (40.0, 4.0), (60.0, 8.0), (80.0, 7.0), (100.0, 9.0)];
        let fahrenheit = vec![(32.0, 3.0), (68.0, 6.0), (104.0, 5.5), (140.0, 7.5), (176.0, 9.5), (212.0, 8.0)]
            .x_scale(FAHRENHEIT)
            .color(DataColor::Color(GREEN));

        Chart::new((
            ScatterController::new(celsius),
            ScatterController::new(fahrenheit),
        ))
            .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).grid_lines(false))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// `two_y_scales_chart` plus units sold on a third scale, also on the right, added after
    /// `percent` so it stacks outside it. Both right-hand scales draw no grid lines, so only Y's
    /// lines cross the chart area; `units` also draws no tick marks, only labels and its border.
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
            .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false))
            .scale(UNITS, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false).tick_marks(false))
            .border().color(BLUE)
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
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }

    /// The chart from `carbide/examples/chart/chart.rs`, with the example's `data7` and `data8`.
    pub fn example_chart(stepped: LocalState<Stepped>) -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0].stepped(stepped);
        let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        Chart::new(ScatterController::new((data7, data8)))
            .border().color(BLUE)
            .locale(ValueState::new(locale!("da")))
    }
}
