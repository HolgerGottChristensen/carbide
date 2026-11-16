//! Category scales: labels, grid lines between categories, and category scales on every axis.

use carbide_chart::{Axis, AxisPosition, BarController, CategoryScale, Chart, DatasetExt, LineController, ScaleId};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::STATUS_LABELS;

/// A bar chart of votes for six colours, with the colours as category labels. The x scale is
/// set explicitly to a `CategoryScale` with labels, so the ticks show "Red", "Blue", ... instead
/// of indices. Pins down label text on a category scale and grid lines between the bars.
#[test]
fn bar_chart_with_labels() {
    // Votes for six colours as bars, labelled on a category scale.
    let votes = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0];

    let chart = Chart::new(BarController::new(votes))
        .scale(
            ScaleId::X,
            CategoryScale::new(Axis::Horizontal)
                .labels(["Red", "Blue", "Yellow", "Green", "Purple", "Orange"])
                .grid_offset(true),
        )
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_with_labels");
}

/// Bars with their category x scale at the top, labelled. Pins down a category scale's labels,
/// tick marks and grid offset on the top side, with bars still growing up from zero at the
/// bottom of the y scale.
#[test]
fn bars_with_x_scale_on_top() {
    // Votes for six colours as bars, with the labelled category x scale at the top.
    let votes = vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0];

    let chart = Chart::new(BarController::new(votes))
        .scale(
            ScaleId::X,
            CategoryScale::new(Axis::Horizontal)
                .position(AxisPosition::Top)
                .labels(["Red", "Blue", "Yellow", "Green", "Purple", "Orange"])
                .grid_offset(true),
        )
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_with_x_scale_on_top");
}

/// A status over time on a labelled vertical category scale with grid offset. Pins down that
/// vertical category scales run from top to bottom (the first label at the top), with labels on
/// the left and grid lines between the categories.
#[test]
fn vertical_category_scale() {
    // A status per day (an index into `STATUS_LABELS`) on a vertical category y scale.
    let status = vec![0.0, 1.0, 1.0, 2.0, 3.0, 2.0, 1.0];

    let chart = Chart::new(LineController::new(status))
        .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(STATUS_LABELS).grid_offset(true))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("vertical_category_scale");
}

/// A temperature line on the default linear y scale, and a status line on a vertical category
/// scale on the right, without grid lines. Pins down a category scale's labels on the right side
/// next to a linear scale on the left.
#[test]
fn vertical_category_scale_on_right() {
    const STATUS: ScaleId = ScaleId::new("status");
    // A temperature on the default y scale, and the status from `vertical_category_scale_chart`
    // on a category scale on the right.
    let temperature = vec![18.0, 21.0, 24.0, 29.0, 34.0, 30.0, 23.0];
    let status = vec![0.0, 1.0, 1.0, 2.0, 3.0, 2.0, 1.0]
        .y_scale(STATUS);

    let chart = Chart::new(LineController::new((temperature, status)))
        .scale(STATUS, CategoryScale::new(Axis::Vertical).position(AxisPosition::Right).labels(STATUS_LABELS).grid_lines(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("vertical_category_scale_on_right");
}

/// Bars on a category scale whose labels and grid offset are states, rendered once as set and
/// once after a label was renamed, a fifth one added and the grid offset turned off. Pins down
/// that the scale names its ticks with the labels as last synced, spans every label, so the new
/// one gets a category without data, and draws its grid lines through the categories once the
/// grid offset is off.
#[test]
fn category_labels_change_between_renders() {
    let mut labels = LocalState::new(STATUS_LABELS.iter().map(|label| label.to_string()).collect::<Vec<String>>());
    let mut grid_offset = LocalState::new(true);

    let chart = Chart::new(BarController::new(vec![4.0, 9.0, 6.0, 2.0]))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(labels.clone()).grid_offset(grid_offset.clone()))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("as_set");

    labels.value_mut()[1] = "Moderate".to_string();
    labels.value_mut().push("Unknown".to_string());
    *grid_offset.value_mut() = false;
    harness.snapshot("changed");
}

// ===============================
//  Utility
// ===============================

mod support {

    pub const STATUS_LABELS: [&str; 4] = ["Low", "Medium", "High", "Critical"];
}
