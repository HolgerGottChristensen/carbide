//! Normalised stacks (`StackId::normalized`): each category's stack scaled to sum to 100, so its
//! parts show as shares of a whole, as 100% stacked bars and areas, along a side or around a
//! circle.

use carbide_chart::{Axis, BarController, CategoryScale, Chart, DatasetExt, Fill, LegendPosition, LineController, ScaleId, StackId};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];

/// 100% stacked bars: three datasets in one normalised stack, with totals that differ from quarter
/// to quarter. Pins down that every bar reaches 100, each part its share of its quarter's total,
/// and that the y scale runs from 0 to 100.
#[test]
fn bars_of_100_percent() {
    const SHARE: StackId = StackId::new("share").normalized();

    let north = vec![30.0, 50.0, 20.0, 45.0].stack(SHARE).label("North");
    let south = vec![50.0, 20.0, 40.0, 15.0].stack(SHARE).label("South");
    let west = vec![20.0, 30.0, 60.0, 40.0].stack(SHARE).label("West");

    let chart = Chart::new(BarController::new((north, south, west)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(QUARTERS))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_of_100_percent");
}

/// Lying 100% stacked bars with a negative value. Pins down that `horizontal` stacks the shares
/// along x; that the absolute values make up the total, so a stack with a negative part splits the
/// 100 between its two sides of zero; and that a category whose values are all zero has no bar.
#[test]
fn lying_bars_with_a_negative_part() {
    const SHARE: StackId = StackId::new("share").normalized();

    let gains = vec![30.0, 60.0, 0.0, 25.0].stack(SHARE).label("Gains");
    let losses = vec![-10.0, 40.0, 0.0, -75.0].stack(SHARE).label("Losses");

    let chart = Chart::new(BarController::new((gains, losses)).horizontal(true))
        .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(QUARTERS))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("lying_bars_with_a_negative_part");
}

/// A 100% stacked area: three filled lines in one normalised stack. Pins down that the top line
/// runs along 100 everywhere, each band its dataset's share at each month, filled down to the line
/// below it.
#[test]
fn area_of_100_percent() {
    const SHARE: StackId = StackId::new("share").normalized();
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

    let lines = (
        vec![30.0, 35.0, 40.0, 30.0, 25.0, 20.0].stack(SHARE).fill(Fill::Bottom).label("Mobile"),
        vec![50.0, 45.0, 40.0, 45.0, 50.0, 50.0].stack(SHARE).fill(Fill::Stack).label("Desktop"),
        vec![20.0, 20.0, 20.0, 25.0, 25.0, 30.0].stack(SHARE).fill(Fill::Stack).label("Tablet"),
    );

    let chart = Chart::new(LineController::new(lines))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(months))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("area_of_100_percent");
}

/// Normalised radial bars: lying bars around a circle, in one normalised stack. Pins down that
/// every ring goes all the way round, as the angle scale fits the stack's 0 to 100 exactly, each
/// arc its share of the ring, so each ring is a doughnut of its own category's parts.
#[test]
fn rings_of_100_percent() {
    const SHARE: StackId = StackId::new("share").normalized();

    let north = vec![30.0, 50.0, 20.0].stack(SHARE).label("North");
    let south = vec![50.0, 20.0, 40.0].stack(SHARE).label("South");
    let west = vec![20.0, 30.0, 60.0].stack(SHARE).label("West");

    let chart = Chart::new(BarController::new((north, south, west)).horizontal(true).category_percentage(1.0))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["2024", "2025", "2026"]))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("rings_of_100_percent");
}
