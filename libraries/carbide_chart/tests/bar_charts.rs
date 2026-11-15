//! Bar charts on their own: bars growing from zero, grouped datasets splitting a slot, and bar
//! widths on a linear x scale.

use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::bar_chart;
use support::bars_on_linear_x_scale_chart;

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

/// One dataset of only negative values. Zero is still included in the y scale, so it is the top
/// of the scale and every bar hangs down from it.
#[test]
fn bar_chart_all_negative() {
    let mut harness = Harness::new(bar_chart(vec![-5.0, -12.0, -8.0, -20.0, -3.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_all_negative");
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

/// Bars at numeric x values (hours of the day) on an explicit linear x scale. Pins down the
/// linear scale's offset of half a tick step at both ends, and bar widths taken from the smallest
/// gap between bars or ticks.
#[test]
fn bars_on_linear_x_scale() {
    let mut harness = Harness::new(bars_on_linear_x_scale_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bars_on_linear_x_scale");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Axis, BarController, Chart, LinearScale, ScaleId};
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    /// Visitors per four-hour block of a day, as bars at numeric x values (the hour) on a
    /// linear x scale.
    pub fn bars_on_linear_x_scale_chart() -> impl Widget {
        let visitors = vec![(0.0, 3.0), (4.0, 1.0), (8.0, 12.0), (12.0, 18.0), (16.0, 15.0), (20.0, 9.0), (24.0, 4.0)];

        Chart::new(BarController::new(visitors))
            .scale(ScaleId::X, LinearScale::new(Axis::Horizontal))
            .framed()
    }
}
