//! Charts whose data, datasets or controllers change after the chart is built.

use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use common::bar_chart;
use support::{controllers_chart, revenue_and_costs_bars, trend_line};

/// Grouped bars whose first dataset is held in a state, changed between the two snapshots: its
/// values grow past the old y range, and a sixth month is appended. Pins down that the chart
/// follows data changed after it is built: the y scale refits to the new values, the category x
/// scale gains a slot for the new month, and the second dataset, which didn't change, keeps its
/// bars in every month it has data for.
#[test]
fn data_changed_between_renders() {
    let mut revenue = LocalState::new(vec![12.0, 19.0, 3.0, 5.0, 2.0]);
    let costs = vec![8.0, 11.0, 6.0, 4.0, 3.0];

    let mut harness = Harness::new(bar_chart((revenue.clone(), costs)), Dimension::new(600.0, 500.0));
    harness.snapshot("before_change");

    *revenue.value_mut() = vec![12.0, 19.0, 30.0, 5.0, 24.0, 17.0];
    harness.snapshot("after_change");
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

    let mut harness = Harness::new(bar_chart(datasets.clone()), Dimension::new(600.0, 500.0));
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
    let bars = revenue_and_costs_bars();
    let mut controllers = LocalState::new(vec![bars.clone()]);

    let mut harness = Harness::new(controllers_chart(controllers.clone()), Dimension::new(600.0, 500.0));
    harness.snapshot("bars_only");

    let trend = trend_line();
    *controllers.value_mut() = vec![bars, trend];
    harness.snapshot("bars_and_line");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{BarController, Chart, DatasetController, LineController};
    use carbide_core::state::LocalState;
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    /// A chart over a state of boxed controllers, so controllers of different types can be added
    /// and removed after it is built.
    pub fn controllers_chart(controllers: LocalState<Vec<Box<dyn DatasetController>>>) -> impl Widget {
        Chart::new(controllers)
            .framed()
    }

    /// Revenue and costs as grouped bars, boxed to share a `Vec` with other controller types.
    pub fn revenue_and_costs_bars() -> Box<dyn DatasetController> {
        BarController::new((vec![12.0, 19.0, 3.0, 5.0, 2.0], vec![8.0, 11.0, 6.0, 4.0, 3.0])).boxed()
    }

    /// A trend line, its colours starting after the bars' two (orange), boxed to share a `Vec`
    /// with other controller types.
    pub fn trend_line() -> Box<dyn DatasetController> {
        LineController::new(vec![10.0, 16.0, 22.0, 18.0, 25.0]).palette_offset(2).boxed()
    }
}
