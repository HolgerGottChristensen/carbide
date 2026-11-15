//! Data whose range has no width, so the scales must widen it themselves.

use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use support::{equal_values_chart, single_point_chart};

/// A single point, so both linear scales start out with no width (min equals max). Pins down that
/// each is widened around the value, as Chart.js does (5% of the value, or 1 at zero), so the
/// point sits inside the chart area with ticks on both sides. It used to panic while rounding
/// the ticks.
#[test]
fn scatter_single_point() {
    let mut harness = Harness::new(single_point_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_single_point");
}

/// A line whose values are all equal, so only the y scale starts out with no width. Pins down
/// that y is widened around the value, putting the flat line in the middle of the chart area,
/// while x keeps its categories.
#[test]
fn line_with_equal_values() {
    let mut harness = Harness::new(equal_values_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("line_with_equal_values");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Chart, LineController, ScatterController};
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    /// One point on linear scales: both scales start out with min equal to max.
    pub fn single_point_chart() -> impl Widget {
        Chart::new(ScatterController::new(vec![(2.0, 1.0)]))
            .framed()
    }

    /// A line through equal values: the y scale starts out with min equal to max.
    pub fn equal_values_chart() -> impl Widget {
        Chart::new(LineController::new(vec![5.0, 5.0, 5.0, 5.0]))
            .framed()
    }
}
