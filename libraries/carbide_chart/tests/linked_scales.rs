//! Scales linked to their datasets: on a chart with two scales running the same way, hovering one
//! scale's band dims the datasets drawn on the other.

use carbide_chart::{Axis, AxisPosition, BarController, Chart, DatasetExt, LinearScale, LineController, ScaleId};
use carbide_core::draw::{Dimension, Position};
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Revenue and costs as bars on the left y scale, and the margin as a line on a second y scale
/// on the right, hovered on each band in turn. Pins down that hovering the right band dims the
/// bars, which aren't on it, and leaves the line; and that hovering the left band dims the line,
/// and leaves both bars.
#[test]
fn hovering_a_scale_dims_the_datasets_on_others() {
    const PERCENT: ScaleId = ScaleId::new("percent");

    let revenue = vec![30.0, 42.0, 38.0, 51.0, 47.0, 60.0];
    let costs = vec![22.0, 30.0, 33.0, 35.0, 31.0, 40.0];
    let margin = vec![27.0, 29.0, 13.0, 31.0, 34.0, 33.0];

    let chart = Chart::new((
        BarController::new((revenue.label("Revenue"), costs.label("Costs"))),
        LineController::new(margin.label("Margin (%)").y_scale(PERCENT)).palette_offset(2),
    ))
        .scale(PERCENT, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).title("Margin (%)"))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));
    // The chart learns where its bands are when it renders.
    harness.render();

    harness.mouse_move(Position::new(555.0, 200.0));
    harness.snapshot("the_right_scale_hovered");

    harness.mouse_move(Position::new(45.0, 200.0));
    harness.snapshot("the_left_scale_hovered");
}
