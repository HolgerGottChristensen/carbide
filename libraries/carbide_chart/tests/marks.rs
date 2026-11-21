//! Marks: floating bars in place of bars and error bars on top of them, in a bar controller's
//! slots, and bubbles in place of a scatter controller's points.

use carbide_chart::{BarController, Bubble, Chart, Estimate, Range, RangeDataSetExt, ScatterController, SizedDataSetExt};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// A month's lowest and highest temperature as a floating bar. Pins down that each bar spans from
/// its low to its high value instead of growing from zero, and that the y scale fits the ranges
/// without reaching down to zero.
#[test]
fn floating_bars() {
    let temperatures = vec![
        Range { x: 0.0, low: 4.0, high: 12.0 },
        Range { x: 1.0, low: 6.0, high: 15.0 },
        Range { x: 2.0, low: 9.0, high: 19.0 },
        Range { x: 3.0, low: 11.0, high: 23.0 },
        Range { x: 4.0, low: 10.0, high: 21.0 },
    ];

    let chart = Chart::new(BarController::new(temperatures.range())).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("floating_bars");
}

/// Two grouped datasets of measurements, each bar with error bars. Pins down that each whisker
/// runs through the middle of its own bar, in its own share of the slot, with caps at both ends,
/// drawn over the bars, and that the y scale reaches the highest whisker.
#[test]
fn grouped_bars_with_error_bars() {
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

    harness.snapshot("grouped_bars_with_error_bars");
}

/// Lying floating bars beside lying bars with error bars, in one controller. Pins down that both
/// marks follow the controller's orientation: the ranges and whiskers run along the x scale, the
/// whiskers' caps across the bars, and each dataset keeps its share of the slot.
#[test]
fn horizontal_marks() {
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

    harness.snapshot("horizontal_marks");
}

/// Countries as bubbles: income along x, life expectancy up y, and population as the bubble's
/// size, in two datasets for two continents. Pins down that a bubble's area is proportional to its
/// size, with the largest size of either dataset at the largest radius, so bubbles compare across
/// datasets; that the bubbles are translucent with an outline, so overlapping ones show through;
/// and that the scales fit the bubbles' centres.
#[test]
fn bubbles() {
    let first = vec![
        Bubble { x: 12.0, y: 71.0, size: 140.0 },
        Bubble { x: 25.0, y: 78.0, size: 35.0 },
        Bubble { x: 40.0, y: 81.0, size: 8.0 },
        Bubble { x: 18.0, y: 74.0, size: 60.0 },
    ];
    let second = vec![
        Bubble { x: 45.0, y: 82.0, size: 10.0 },
        Bubble { x: 30.0, y: 80.0, size: 20.0 },
        Bubble { x: 22.0, y: 76.0, size: 2.0 },
    ];

    let chart = Chart::new(ScatterController::new((first.bubbles(), second.bubbles()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubbles");
}
