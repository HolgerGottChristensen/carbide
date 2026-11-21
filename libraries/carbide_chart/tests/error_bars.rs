//! Error bars: whiskers from each point's low to its high value, on bars and on lines, moving
//! with stacks.

use carbide_chart::{BarController, Chart, DatasetExt, Estimate, Interpolation, LineController, Range, RangeDataSetExt, StackId};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Two grouped datasets of measurements, each bar with error bars. Pins down that each whisker
/// runs through the middle of its own bar, in its own share of the slot, with caps at both ends,
/// drawn over the bars, and that the y scale reaches the highest whisker.
#[test]
fn on_grouped_bars() {
    let control = vec![
        Estimate { x: 0.0, y: 12.0, low: 10.0, high: 15.0 },
        Estimate { x: 1.0, y: 18.0, low: 14.0, high: 21.0 },
        Estimate { x: 2.0, y: 9.0, low: 8.0, high: 11.0 },
    ];
    let treated = vec![
        Estimate { x: 0.0, y: 15.0, low: 12.0, high: 17.0 },
        Estimate { x: 1.0, y: 24.0, low: 20.0, high: 29.0 },
        Estimate { x: 2.0, y: 13.0, low: 9.0, high: 16.0 },
    ];

    let chart = Chart::new(BarController::new((control.error_bars(), treated.error_bars()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_grouped_bars");
}

/// Two stacked datasets with error bars, in months with positive and with negative values. Pins
/// down that each whisker moves with its stacked bar: it keeps its distance from the dataset's
/// value, around where its bar ends, upwards above zero and downwards below it, and that the y
/// scale reaches the highest and lowest whisker.
#[test]
fn on_stacked_bars() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![
        Estimate { x: 0.0, y: 10.0, low: 8.0, high: 12.0 },
        Estimate { x: 1.0, y: 14.0, low: 12.0, high: 15.0 },
        Estimate { x: 2.0, y: -8.0, low: -10.0, high: -7.0 },
    ];
    let labour = vec![
        Estimate { x: 0.0, y: 6.0, low: 4.0, high: 9.0 },
        Estimate { x: 1.0, y: 4.0, low: 3.0, high: 6.0 },
        Estimate { x: 2.0, y: -5.0, low: -7.0, high: -4.0 },
    ];

    let chart = Chart::new(BarController::new((materials.error_bars().stack(COSTS), labour.error_bars().stack(COSTS)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_stacked_bars");
}

/// Two lines with error bars, one of them monotone and missing a value. Pins down that each
/// whisker runs through its point from the low to the high value, with caps about as wide as a
/// point, drawn over the lines and points, and that the missing value has no whisker.
#[test]
fn on_lines() {
    let first: Vec<Estimate<f64, Option<f64>>> = vec![
        Estimate { x: 0.0, y: Some(12.0), low: Some(10.0), high: Some(15.0) },
        Estimate { x: 1.0, y: Some(18.0), low: Some(15.0), high: Some(20.0) },
        Estimate { x: 2.0, y: None, low: Some(14.0), high: Some(19.0) },
        Estimate { x: 3.0, y: Some(21.0), low: Some(18.0), high: Some(25.0) },
        Estimate { x: 4.0, y: Some(17.0), low: Some(15.0), high: Some(19.0) },
    ];
    let second = vec![
        Estimate { x: 0.0, y: 6.0, low: 4.0, high: 7.0 },
        Estimate { x: 1.0, y: 9.0, low: 7.0, high: 12.0 },
        Estimate { x: 2.0, y: 8.0, low: 6.0, high: 9.0 },
        Estimate { x: 3.0, y: 11.0, low: 9.0, high: 14.0 },
        Estimate { x: 4.0, y: 13.0, low: 11.0, high: 15.0 },
    ];

    let chart = Chart::new(LineController::new((first.error_bars(), second.error_bars().interpolation(Interpolation::Monotone)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_lines");
}

/// Two stacked lines with error bars. Pins down that the upper line's whiskers move with it,
/// around where it is drawn at the running total, keeping their distance from its own value.
#[test]
fn on_stacked_lines() {
    const ALL: StackId = StackId::new("all");

    let base = vec![
        Estimate { x: 0.0, y: 10.0, low: 8.0, high: 11.0 },
        Estimate { x: 1.0, y: 12.0, low: 10.0, high: 14.0 },
        Estimate { x: 2.0, y: 11.0, low: 10.0, high: 13.0 },
        Estimate { x: 3.0, y: 14.0, low: 12.0, high: 15.0 },
    ];
    let extra = vec![
        Estimate { x: 0.0, y: 5.0, low: 4.0, high: 7.0 },
        Estimate { x: 1.0, y: 6.0, low: 4.0, high: 8.0 },
        Estimate { x: 2.0, y: 8.0, low: 6.0, high: 9.0 },
        Estimate { x: 3.0, y: 7.0, low: 5.0, high: 10.0 },
    ];

    let chart = Chart::new(LineController::new((base.error_bars().stack(ALL), extra.error_bars().stack(ALL)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_stacked_lines");
}

/// Lying floating bars beside lying bars with error bars, in one controller. Pins down that both
/// marks follow the controller's orientation: the ranges and whiskers run along the x scale, the
/// whiskers' caps across the bars, and each dataset keeps its share of the slot.
#[test]
fn horizontal() {
    let ranges = vec![
        Range { x: 0.0, low: 3.0, high: 9.0 },
        Range { x: 1.0, low: 5.0, high: 14.0 },
        Range { x: 2.0, low: 2.0, high: 7.0 },
    ];
    let measured = vec![
        Estimate { x: 0.0, y: 8.0, low: 6.0, high: 11.0 },
        Estimate { x: 1.0, y: 12.0, low: 10.0, high: 16.0 },
        Estimate { x: 2.0, y: 5.0, low: 4.0, high: 7.0 },
    ];

    let chart = Chart::new(BarController::new((ranges.range(), measured.error_bars())).horizontal()).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal");
}

/// Marks with missing values: bars with error bars where one value, one low and one high are
/// missing, beside floating bars where one low and one high are missing. Pins down that a missing
/// value has neither a bar nor a whisker, that a missing low or high takes away only the whisker,
/// and that a floating bar missing either end isn't drawn, leaving its place in the slot empty.
#[test]
fn with_missing_values() {
    let measured: Vec<Estimate<f64, Option<f64>>> = vec![
        Estimate { x: 0.0, y: Some(10.0), low: Some(8.0), high: Some(13.0) },
        Estimate { x: 1.0, y: None, low: Some(9.0), high: Some(14.0) },
        Estimate { x: 2.0, y: Some(12.0), low: None, high: Some(15.0) },
        Estimate { x: 3.0, y: Some(7.0), low: Some(5.0), high: None },
    ];
    let ranges: Vec<Range<f64, Option<f64>>> = vec![
        Range { x: 0.0, low: Some(3.0), high: Some(9.0) },
        Range { x: 1.0, low: Some(4.0), high: Some(11.0) },
        Range { x: 2.0, low: None, high: Some(10.0) },
        Range { x: 3.0, low: Some(2.0), high: None },
    ];

    let chart = Chart::new(BarController::new((measured.error_bars(), ranges.range()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("with_missing_values");
}
