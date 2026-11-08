use carbide_chart::Stepped;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

use support::example_chart;

/// Renders the chart from `carbide/examples/chart/chart.rs` (two datasets, the first one's
/// `stepped` mode driven by a state, Danish locale, bordered) and snapshots it once per `Stepped`
/// variant, the same sequence the example's picker steps through. The datasets are fixed, so
/// each snapshot pins down the linear scale's ticks and grid, the tick labels, the dataset
/// colors and the line path for one stepped mode.
#[test]
fn scatter_chart_stepped_modes() {
    let stepped = LocalState::new(Stepped::None);
    let mut harness = Harness::new(example_chart(stepped.clone()), Dimension::new(400.0, 300.0));

    for (mode, name) in [
        (Stepped::None, "stepped_none"),
        (Stepped::Before, "stepped_before"),
        (Stepped::After, "stepped_after"),
        (Stepped::Middle, "stepped_middle"),
        (Stepped::MiddleVertical, "stepped_middle_vertical"),
    ] {
        stepped.clone().set_value(mode);
        harness.snapshot(name);
    }
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Chart, DatasetExt, ScatterController, Stepped};
    use carbide_core::state::{LocalState, ValueState};
    use carbide_core::widget::{Widget, WidgetExt};
    use carbide_fluent::{locale, LocaleExt};

    /// The chart from `carbide/examples/chart/chart.rs`, with the example's `data7` and `data8`.
    pub fn example_chart(stepped: LocalState<Stepped>) -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0].stepped(stepped);
        let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        Chart::new(ScatterController::new((data7, data8)))
            .border()
            .locale(ValueState::new(locale!("da")))
    }
}
