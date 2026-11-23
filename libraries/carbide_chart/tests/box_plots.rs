//! Box plots: a box from the first to the third quartile, the median across it, and whiskers
//! out to the minimum and the maximum, in a bar controller's slots.

use carbide_chart::{BarController, BoxDataSetExt, BoxStats, Chart};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Two grouped datasets of box plots over three categories. Pins down that each box runs from the
/// first to the third quartile with the median across it, that the whiskers reach the minimum and
/// the maximum and end in caps half the box wide, that the two datasets sit side by side in each
/// slot, and that the y scale fits the whiskers without reaching down to zero.
#[test]
fn grouped() {
    let before = vec![
        BoxStats { x: 0.0, min: 12.0, q1: 18.0, median: 22.0, q3: 27.0, max: 35.0 },
        BoxStats { x: 1.0, min: 15.0, q1: 20.0, median: 25.0, q3: 29.0, max: 38.0 },
        BoxStats { x: 2.0, min: 10.0, q1: 16.0, median: 19.0, q3: 24.0, max: 31.0 },
    ];
    let after = vec![
        BoxStats { x: 0.0, min: 9.0, q1: 13.0, median: 16.0, q3: 20.0, max: 26.0 },
        BoxStats { x: 1.0, min: 11.0, q1: 14.0, median: 18.0, q3: 22.0, max: 30.0 },
        BoxStats { x: 2.0, min: 8.0, q1: 11.0, median: 13.0, q3: 17.0, max: 23.0 },
    ];

    let chart = Chart::new(BarController::new((before.box_plot(), after.box_plot()))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("grouped");
}

/// Horizontal box plots. Pins down that the boxes, medians, whiskers and caps follow the controller's
/// orientation: the categories down the y scale, the values along the x scale, the caps across.
#[test]
fn horizontal() {
    let response_times = vec![
        BoxStats { x: 0.0, min: 80.0, q1: 120.0, median: 150.0, q3: 190.0, max: 260.0 },
        BoxStats { x: 1.0, min: 60.0, q1: 90.0, median: 110.0, q3: 140.0, max: 200.0 },
        BoxStats { x: 2.0, min: 100.0, q1: 160.0, median: 210.0, q3: 250.0, max: 340.0 },
    ];

    let chart = Chart::new(BarController::new(response_times.box_plot()).horizontal(true)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal");
}
