//! Where scales go: scales on each side of the chart area, and several scales stacked on one side.

use carbide_chart::{Axis, AxisPosition, Chart, DatasetExt, LineController, LinearScale, ScaleId, ScatterController};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::{FAHRENHEIT, PERCENT, UNITS};

/// Two controllers on one chart: revenue on the default Y scale (left), and a margin line bound
/// to a custom `percent` scale on the right. Pins down that each dataset is drawn on its own
/// scale, that the right-hand scale gets its ticks, labels and border on the right, and that the
/// chart area makes room for it.
#[test]
fn two_y_scales() {
    // Revenue on the default scales and a margin (as a fraction) on a `percent` scale on the
    // right, in two controllers.
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
    let margin = vec![0.12, 0.18, 0.15, 0.08, 0.22, 0.25, 0.31]
        .y_scale(PERCENT);

    let chart = Chart::new((
        LineController::new(revenue),
        LineController::new(margin).palette_offset(1),
    ))
        .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("two_y_scales");
}

/// The example chart with its default x scale moved to the top. Pins down a horizontal scale's
/// top side: tick marks reaching up from the chart area, labels above them, its border along the
/// top edge, its band taken off the top of the chart area, and no band left at the bottom.
#[test]
fn x_scale_on_top() {
    // The example's two datasets, with the default x scale replaced by one at the top.
    let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0];
    let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

    let chart = Chart::new(LineController::new((data7, data8)))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("x_scale_on_top");
}

/// Two scales on the right: the first added sits against the chart area, the second in its own band
/// outside it. Pins down that neither scale's ticks, labels or border overlap the other's, and that
/// the chart area gives up both bands.
#[test]
fn stacked_scales_on_one_side() {
    // `two_y_scales_chart` plus units sold on a third scale, also on the right, added after
    // `percent` so it stacks outside it. Both right-hand scales draw no grid lines, so only Y's
    // lines cross the chart area; `units` also draws no tick marks, only labels and its border.
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
    let margin = vec![0.12, 0.18, 0.15, 0.08, 0.22, 0.25, 0.31]
        .y_scale(PERCENT);
    let units = vec![1800.0, 2100.0, 2600.0, 1500.0, 3000.0, 3400.0, 3900.0]
        .y_scale(UNITS);

    let chart = Chart::new((
        LineController::new(revenue),
        LineController::new(margin).palette_offset(1),
        LineController::new(units).palette_offset(2),
    ))
        .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false))
        .scale(UNITS, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false).tick_marks(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_one_side");
}

/// Two vertical scales on the left: revenue on the default y scale against the chart area, units
/// sold on a second scale stacked outside it with spacing between their bands.
#[test]
fn stacked_scales_on_left() {
    // Revenue on the default y scale and units sold on a second scale, also on the left, added
    // after Y so it stacks outside it.
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
    let units = vec![1800.0, 2100.0, 2600.0, 1500.0, 3000.0, 3400.0, 3900.0]
        .y_scale(UNITS);

    let chart = Chart::new((
        LineController::new(revenue),
        LineController::new(units).palette_offset(1),
    ))
        .scale(UNITS, LinearScale::new(Axis::Vertical).grid_lines(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_left");
}

/// The horizontal counterpart of `stacked_scales_on_one_side`: a second x scale stacked below the
/// default one, with spacing between their bands. Pins down that horizontal bands stack
/// downwards, each with its own tick marks, labels and border, and that the chart area gives up
/// the height of both bands.
#[test]
fn stacked_horizontal_scales() {
    // Two series of temperature readings, one measured in °C on the default x scale and one in
    // °F on a second x scale, which is also at the bottom and so stacks below the default one.
    // The °F scale draws no grid lines, so only the °C scale's lines cross the chart area.
    let celsius = vec![(0.0, 2.0), (20.0, 5.0), (40.0, 4.0), (60.0, 8.0), (80.0, 7.0), (100.0, 9.0)];
    let fahrenheit = vec![(32.0, 3.0), (68.0, 6.0), (104.0, 5.5), (140.0, 7.5), (176.0, 9.5), (212.0, 8.0)]
        .x_scale(FAHRENHEIT);

    let chart = Chart::new((
        ScatterController::new(celsius).show_line(true),
        ScatterController::new(fahrenheit).show_line(true).palette_offset(1),
    ))
        .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).grid_lines(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_horizontal_scales");
}

/// The temperature chart from `stacked_horizontal_scales`, with both x scales at the top: the
/// default x scale against the chart area, the °F scale stacked above it with spacing between
/// their bands.
#[test]
fn stacked_scales_on_top() {
    // `stacked_horizontal_scales_chart` with both x scales at the top. X is added first (it is a
    // default scale, replaced in place), so it stays against the chart area.
    let celsius = vec![(0.0, 2.0), (20.0, 5.0), (40.0, 4.0), (60.0, 8.0), (80.0, 7.0), (100.0, 9.0)];
    let fahrenheit = vec![(32.0, 3.0), (68.0, 6.0), (104.0, 5.5), (140.0, 7.5), (176.0, 9.5), (212.0, 8.0)]
        .x_scale(FAHRENHEIT);

    let chart = Chart::new((
        ScatterController::new(celsius).show_line(true),
        ScatterController::new(fahrenheit).show_line(true).palette_offset(1),
    ))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top))
        .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top).grid_lines(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_scales_on_top");
}

/// A scale on every side of the chart area at once: °C along the bottom and humidity on the left
/// for one dataset, °F along the top and pressure on the right for the other. Pins down that each
/// side's band is laid out independently and the chart area gives up all four.
#[test]
fn scales_on_all_four_sides() {
    const PRESSURE: ScaleId = ScaleId::new("pressure");
    // Humidity by temperature in °C on the default scales (bottom and left), and air pressure by
    // temperature in °F on scales at the top and on the right.
    let humidity = vec![(0.0, 82.0), (20.0, 70.0), (40.0, 64.0), (60.0, 55.0), (80.0, 48.0), (100.0, 40.0)];
    let pressure = vec![(32.0, 1002.0), (68.0, 1008.0), (104.0, 1011.0), (140.0, 1016.0), (176.0, 1021.0), (212.0, 1026.0)]
        .x_scale(FAHRENHEIT)
        .y_scale(PRESSURE);

    let chart = Chart::new((
        ScatterController::new(humidity).show_line(true),
        ScatterController::new(pressure).show_line(true).palette_offset(1),
    ))
        .scale(FAHRENHEIT, LinearScale::new(Axis::Horizontal).position(AxisPosition::Top).grid_lines(false))
        .scale(PRESSURE, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("scales_on_all_four_sides");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::ScaleId;

    pub const PERCENT: ScaleId = ScaleId::new("percent");
    pub const UNITS: ScaleId = ScaleId::new("units");
    pub const FAHRENHEIT: ScaleId = ScaleId::new("fahrenheit");
}
