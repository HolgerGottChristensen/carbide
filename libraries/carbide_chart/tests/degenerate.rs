//! Data with nothing to span: ranges with no width, which the scales must widen themselves, and
//! charts with no data at all.

use carbide_chart::{BarController, Chart, DatasetController, DatasetExt, Fill, LineController, ScatterController, StackId};
use carbide_core::draw::{Dimension, Scalar};
use carbide_core::state::LocalState;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Charts with nothing to draw: one without controllers, and one whose bar and line controllers
/// hold only empty datasets, stacked and filled. Pins down that both render without panicking,
/// with the default linear scales over `0..1` and nothing in the chart area, and that empty
/// datasets ask nothing of the default scales but the kind (the bars' empty dataset still makes
/// x a category scale).
#[test]
fn empty() {
    // A chart over a state holding no controllers.
    let controllers: LocalState<Vec<Box<dyn DatasetController>>> = LocalState::new(vec![]);

    let chart = Chart::new(controllers)
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("no_controllers");

    // Bars and lines with only empty datasets, the lines stacked and filled.
    const STACK: StackId = StackId::new("stack");
    let empty: Vec<Scalar> = vec![];

    let chart = Chart::new((
        BarController::new((empty.clone().stack(STACK), empty.clone())),
        LineController::new((empty.clone().stack(STACK).fill(Fill::Stack), empty.fill(Fill::Origin))),
    ))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("empty_datasets");
}

/// A single point, so both linear scales start out with no width (min equals max). Pins down that
/// each is widened around the value (by 5% of the value, or 1 at zero), so the point sits inside
/// the chart area with ticks on both sides. It used to panic while rounding the ticks.
#[test]
fn scatter_single_point() {
    // One point on linear scales: both scales start out with min equal to max.
    let chart = Chart::new(ScatterController::new(vec![(2.0, 1.0)]))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_single_point");
}

/// A line whose values are all equal, so only the y scale starts out with no width. Pins down
/// that y is widened around the value, putting the flat line in the middle of the chart area,
/// while x keeps its categories.
#[test]
fn line_with_equal_values() {
    // A line through equal values: the y scale starts out with min equal to max.
    let chart = Chart::new(LineController::new(vec![5.0, 5.0, 5.0, 5.0]))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("line_with_equal_values");
}
