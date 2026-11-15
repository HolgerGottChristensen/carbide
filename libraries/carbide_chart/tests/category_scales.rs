//! Category scales: labels, grid lines between categories, and category scales on every axis.

use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use support::{bars_with_x_scale_on_top_chart, labelled_bar_chart, vertical_category_scale_chart, vertical_category_scale_on_right_chart};

/// Chart.js's sample bar chart, with its category labels: votes for six colours. The x scale is
/// set explicitly to a `CategoryScale` with labels, so the ticks show "Red", "Blue", ... instead
/// of indices. Pins down label text on a category scale and grid lines between the bars.
#[test]
fn bar_chart_with_labels() {
    let mut harness = Harness::new(labelled_bar_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_with_labels");
}

/// Bars with their category x scale at the top, labelled. Pins down a category scale's labels,
/// tick marks and grid offset on the top side, with bars still growing up from zero at the
/// bottom of the y scale.
#[test]
fn bars_with_x_scale_on_top() {
    let mut harness = Harness::new(bars_with_x_scale_on_top_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bars_with_x_scale_on_top");
}

/// A status over time on a labelled vertical category scale with grid offset. Pins down that
/// vertical category scales run from top to bottom (the first label at the top), with labels on
/// the left and grid lines between the categories.
#[test]
fn vertical_category_scale() {
    let mut harness = Harness::new(vertical_category_scale_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("vertical_category_scale");
}

/// A temperature line on the default linear y scale, and a status line on a vertical category
/// scale on the right, without grid lines. Pins down a category scale's labels on the right side
/// next to a linear scale on the left.
#[test]
fn vertical_category_scale_on_right() {
    let mut harness = Harness::new(vertical_category_scale_on_right_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("vertical_category_scale_on_right");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Axis, AxisPosition, BarController, CategoryScale, Chart, DataColor, DatasetExt, LineController, ScaleId};
    use carbide_core::color::RED;
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    const STATUS: ScaleId = ScaleId::new("status");

    const STATUS_LABELS: [&str; 4] = ["Low", "Medium", "High", "Critical"];

    /// Chart.js's sample bar chart: votes for six colours, labelled on a category scale.
    pub fn labelled_bar_chart() -> impl Widget {
        let votes = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0];

        Chart::new(BarController::new(votes))
            .scale(
                ScaleId::X,
                CategoryScale::new(Axis::Horizontal)
                    .labels(["Red", "Blue", "Yellow", "Green", "Purple", "Orange"])
                    .grid_offset(true),
            )
            .framed()
    }

    /// Chart.js's sample votes as bars, with the labelled category x scale at the top.
    pub fn bars_with_x_scale_on_top_chart() -> impl Widget {
        let votes = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0];

        Chart::new(BarController::new(votes))
            .scale(
                ScaleId::X,
                CategoryScale::new(Axis::Horizontal)
                    .position(AxisPosition::Top)
                    .labels(["Red", "Blue", "Yellow", "Green", "Purple", "Orange"])
                    .grid_offset(true),
            )
            .framed()
    }

    /// A status per day (an index into `STATUS_LABELS`) on a vertical category y scale.
    pub fn vertical_category_scale_chart() -> impl Widget {
        let status = vec![0.0, 1.0, 1.0, 2.0, 3.0, 2.0, 1.0];

        Chart::new(LineController::new(status))
            .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(STATUS_LABELS).grid_offset(true))
            .framed()
    }

    /// A temperature on the default y scale, and the status from `vertical_category_scale_chart`
    /// on a category scale on the right.
    pub fn vertical_category_scale_on_right_chart() -> impl Widget {
        let temperature = vec![18.0, 21.0, 24.0, 29.0, 34.0, 30.0, 23.0];
        let status = vec![0.0, 1.0, 1.0, 2.0, 3.0, 2.0, 1.0]
            .y_scale(STATUS)
            .color(DataColor::Color(RED));

        Chart::new(LineController::new((temperature, status)))
            .scale(STATUS, CategoryScale::new(Axis::Vertical).position(AxisPosition::Right).labels(STATUS_LABELS).grid_lines(false))
            .framed()
    }
}
