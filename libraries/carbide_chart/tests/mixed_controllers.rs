//! Several controllers in one chart: sharing scales, draw order, and how their requests to the
//! scales combine.

use carbide_chart::{Axis, AxisPosition, BarController, Chart, DatasetExt, LineController, LinearScale, ScaleId};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Bars and a line in one chart, on the same scales: revenue and costs as grouped bars from a
/// `BarController`, profit as a line from a `LineController` drawn over them. Pins down that
/// the two bar datasets sit side by side around each x value, that the first and last groups
/// aren't cut off at the chart area's edges (the x scale's offset), that bars grow from zero,
/// and that the line's points sit at the centre of each group, including the negative month.
#[test]
fn bars_and_line_on_shared_scales() {
    // Revenue and costs as grouped bars, and profit (revenue minus costs, negative in one month)
    // as a line over them, its colours starting after the bars' two, all on the default scales.
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
    let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0, 150.0, 155.0];
    let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0, 25.0, 35.0];

    let chart = Chart::new((
        BarController::new((revenue, costs)),
        LineController::new(profit).palette_offset(2),
    ))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_and_line_on_shared_scales");
}

/// The data of `bars_and_line_on_shared_scales` with the controllers the other way round: the
/// line controller first. Pins down that the bars' requests still reach the shared x scale: grid
/// lines sit between the bar groups and the scale is offset, as with the bars first. Only the
/// layering changes: the line is drawn first, under the bars.
#[test]
fn line_then_bars() {
    // `bars_and_line_chart` with the line controller before the bar controller.
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
    let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0, 150.0, 155.0];
    let profit = vec![20.0, 15.0, 25.0, -20.0, 20.0, 25.0, 35.0];

    let chart = Chart::new((
        LineController::new(profit).palette_offset(2),
        BarController::new((revenue, costs)),
    ))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("line_then_bars");
}

/// Two bar controllers on the same x scale. Bars are grouped only within one controller, so
/// separate controllers overlap: the second controller's bars are drawn over the first's in the
/// same slot, like the layers of a `ZStack`. This is intended; use one controller with several
/// datasets for side-by-side bars.
#[test]
fn two_bar_controllers_on_one_x_scale() {
    // Revenue and costs as bars in two separate bar controllers on the default scales.
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];
    let costs = vec![100.0, 120.0, 125.0, 130.0, 140.0, 150.0, 155.0];

    let chart = Chart::new((
        BarController::new(revenue),
        BarController::new(costs).palette_offset(1),
    ))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("two_bar_controllers_on_one_x_scale");
}

/// Bars on a secondary y scale: margin as bars on a `percent` scale on the right, revenue as a
/// line on the default y scale on the left. Pins down that bars grow from zero on the scale they
/// are bound to, whose range is stretched to include zero, while the line's scale is not.
#[test]
fn bars_on_secondary_y_scale() {
    const PERCENT: ScaleId = ScaleId::new("percent");
    // Margin as bars on `percent` on the right, revenue as a line on Y on the left.
    let margin = vec![0.12, 0.18, 0.15, 0.08, 0.22, 0.25, 0.31]
        .y_scale(PERCENT)
;
    let revenue = vec![120.0, 135.0, 150.0, 110.0, 160.0, 175.0, 190.0];

    let chart = Chart::new((
        BarController::new(margin),
        LineController::new(revenue).palette_offset(1),
    ))
        .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_on_secondary_y_scale");
}
