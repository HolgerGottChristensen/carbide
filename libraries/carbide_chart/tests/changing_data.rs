//! Charts whose data, datasets or controllers change after the chart is built.

use carbide_chart::{BarController, Chart, DatasetController, DatasetExt, LineController, StackId};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use common::Framed;

/// Grouped bars whose first dataset is held in a state, changed between the two snapshots: its
/// values grow past the old y range, and a sixth month is appended. Pins down that the chart
/// follows data changed after it is built: the y scale refits to the new values, the category x
/// scale gains a slot for the new month, and the second dataset, which didn't change, keeps its
/// bars in every month it has data for.
#[test]
fn data_changed_between_renders() {
    let mut revenue = LocalState::new(vec![12.0, 19.0, 3.0, 5.0, 2.0]);
    let costs = vec![8.0, 11.0, 6.0, 4.0, 3.0];

    let chart = Chart::new(BarController::new((revenue.clone(), costs))).framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("before_change");

    *revenue.value_mut() = vec![12.0, 19.0, 30.0, 5.0, 24.0, 17.0];
    harness.snapshot("after_change");
}

/// Grouped bars whose controller options are states, made horizontal and narrowed between the two
/// snapshots. Pins down that a controller's options follow their states after the chart is built:
/// the categories move to the y scale and the values to the x scale, and each bar takes less of
/// its share of the slot.
#[test]
fn controller_options_changed_between_renders() {
    let mut horizontal = LocalState::new(false);
    let mut bar_percentage = LocalState::new(0.9);

    let chart = Chart::new(BarController::new((vec![12.0, 19.0, 3.0, 5.0], vec![8.0, 11.0, 6.0, 4.0]))
        .horizontal(horizontal.clone())
        .bar_percentage(bar_percentage.clone()))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("vertical");

    *horizontal.value_mut() = true;
    *bar_percentage.value_mut() = 0.5;
    harness.snapshot("horizontal_and_narrower");
}

/// Grouped bars whose datasets are a state of a `Vec`, with a third dataset added between the two
/// snapshots. Pins down that the chart follows a dataset sequence changed after it is built: each
/// month's slot splits three ways instead of two, the new dataset takes the next palette colour
/// (orange), and the y scale refits to include its larger values.
#[test]
fn dataset_added_between_renders() {
    let revenue = vec![12.0, 19.0, 3.0, 5.0, 2.0];
    let costs = vec![8.0, 11.0, 6.0, 4.0, 3.0];
    let mut datasets = LocalState::new(vec![revenue.clone(), costs.clone()]);

    let chart = Chart::new(BarController::new(datasets.clone())).framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("two_datasets");

    *datasets.value_mut() = vec![revenue, costs, vec![15.0, 24.0, 9.0, 7.0, 4.0]];
    harness.snapshot("three_datasets");
}

/// A chart whose controllers are a state of boxed controllers of different types, with a line
/// added over the bars between the two snapshots. Pins down that the chart follows a controller
/// sequence changed after it is built: the line is drawn over the bars at the bars' centres, the
/// y scale refits to include its larger values, and the x scale stays a category scale with grid
/// lines between the bar groups, as both controllers ask for a category scale.
#[test]
fn controllers_added_between_renders() {
    // Revenue and costs as grouped bars, boxed to share a `Vec` with other controller types.
    let bars = BarController::new((vec![12.0, 19.0, 3.0, 5.0, 2.0], vec![8.0, 11.0, 6.0, 4.0, 3.0])).boxed();
    let mut controllers = LocalState::new(vec![bars.clone()]);


    // A chart over the state of boxed controllers, so controllers of different types can be added
    // and removed after it is built.
    let chart = Chart::new(controllers.clone())
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("bars_only");

    // A trend line, its colours starting after the bars' two (orange), boxed to share a `Vec`
    // with other controller types.
    let trend = LineController::new(vec![10.0, 16.0, 22.0, 18.0, 25.0]).palette_offset(2).boxed();
    *controllers.value_mut() = vec![bars, trend];
    harness.snapshot("bars_and_line");
}

/// Grouped bars whose second dataset's stack is a state, changed between the two snapshots: it
/// starts in a stack of its own, beside the first dataset, and is then moved into the first
/// dataset's stack. Pins down that the chart follows a stack changed after it is built: each
/// month goes from two bars side by side to one stacked bar, and the y scale refits to reach the
/// stacked totals.
#[test]
fn stack_changed_between_renders() {
    const COSTS: StackId = StackId::new("costs");

    let mut stack = LocalState::new(StackId::new("labour"));
    let materials = vec![40.0, 45.0, 50.0, 42.0, 55.0].stack(COSTS);
    let labour = vec![60.0, 62.0, 70.0, 65.0, 80.0].stack(stack.clone());

    let chart = Chart::new(BarController::new((materials, labour))).framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("apart");

    *stack.value_mut() = COSTS;
    harness.snapshot("stacked");
}
