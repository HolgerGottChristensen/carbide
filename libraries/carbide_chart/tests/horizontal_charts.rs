//! Bar and line charts lying down: the same data as upright charts, with the categories down the
//! y scale and the values along the x scale.

use carbide_chart::{Axis, BarController, CategoryScale, Chart, DatasetExt, Fill, Interpolation, LineController, LinearScale, ScaleId, StackId, Stepped};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Two datasets with positive and negative values as horizontal bars, on the default scales. Pins
/// down that the categories run down a vertical category scale, the first at the top, with grid
/// lines between the groups and half a step of offset at both ends; that each group splits its
/// slot with the first dataset on top; and that the bars grow from zero along the x scale, right
/// for positive values and left for negative ones.
#[test]
fn horizontal_bars() {
    let revenue = vec![12.0, 19.0, -3.0, 5.0, 2.0];
    let costs = vec![8.0, -11.0, 6.0, 4.0, 3.0];

    let chart = Chart::new(BarController::new((revenue, costs)).horizontal())
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_bars");
}

/// Two stacked datasets with positive and negative values and one dataset on its own, as
/// horizontal bars. Pins down that a stack lies along the x scale, positive values stacking right
/// of zero and negative ones left, each on their own, the first dataset nearest zero; and that the
/// stack and the dataset on its own split each slot, the stack on top.
#[test]
fn horizontal_stacked_bars() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![10.0, 20.0, 15.0, 8.0].stack(COSTS);
    let labour = vec![5.0, -12.0, 6.0, 9.0].stack(COSTS);
    let revenue = vec![18.0, 14.0, -6.0, 20.0];

    let chart = Chart::new(BarController::new((materials, labour, revenue)).horizontal())
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_stacked_bars");
}

/// Horizontal bars on a vertical category scale with labels, and a titled value scale. Pins down
/// that a category scale set with `Chart::scale` names the rows from the top, and still gets the
/// bars' offset and grid lines between the rows.
#[test]
fn horizontal_bars_with_labels() {
    let visitors = vec![1200.0, 950.0, 640.0, 410.0, 180.0];

    let chart = Chart::new(BarController::new(visitors).horizontal())
        .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(["Search", "Direct", "Social", "Email", "Referral"]))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).title("Visitors"))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_bars_with_labels");
}

/// Revenue as horizontal bars with a target line laid down over them. Pins down that a horizontal
/// line puts each point on the same row as the bar for its category, through the bars' centres,
/// and that its values run along the same x scale as the bars.
#[test]
fn horizontal_bars_and_line() {
    let revenue = vec![12.0, 19.0, 8.0, 15.0, 10.0];
    let target = vec![10.0, 15.0, 12.0, 14.0, 16.0];

    let chart = Chart::new((
        BarController::new(revenue).horizontal(),
        LineController::new(target).horizontal().palette_offset(1),
    ))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_bars_and_line");
}

/// A monotone line filled to zero and a stepped line, both laid down. Pins down that what follows
/// the categories follows them down the chart: the monotone curve never overshoots between two
/// rows, the steps change value halfway between rows, and the area fills sideways, between the
/// line and zero on the x scale, left of zero where the values are negative.
#[test]
fn horizontal_lines() {
    let margin = vec![8.0, 12.0, -4.0, 6.0, 14.0, 10.0]
        .interpolation(Interpolation::Monotone)
        .fill(Fill::Origin);
    let budget = vec![5.0, 5.0, 9.0, 9.0, 12.0, 12.0]
        .interpolation(Interpolation::Stepped(Stepped::Middle));

    let chart = Chart::new(LineController::new((margin, budget)).horizontal())
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_lines");
}

/// Three monotone lines in one stack, filled as a stacked area chart, laid down. Pins down that
/// each line is drawn at the running total along the x scale, and that each area fills exactly
/// between its line and the line before it in the stack, sideways.
#[test]
fn horizontal_stacked_areas() {
    const VISITS: StackId = StackId::new("visits");

    let search = vec![30.0, 35.0, 32.0, 40.0, 45.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);
    let social = vec![10.0, 15.0, 8.0, 12.0, 20.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);
    let email = vec![12.0, 8.0, 15.0, 10.0, 14.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);

    let chart = Chart::new(LineController::new((search, social, email)).horizontal())
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_stacked_areas");
}
