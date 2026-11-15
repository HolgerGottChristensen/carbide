//! Line and scatter charts: points, lines through them, and stepped lines.

use carbide_chart::Stepped;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use support::{example_chart, scatter_chart, scatter_with_lines_chart};

/// Renders the chart from `carbide/examples/chart/chart.rs` (two datasets, the first one's
/// `stepped` mode driven by a state, Danish locale, bordered) and snapshots it once per `Stepped`
/// variant, the same sequence the example's picker steps through. The datasets are fixed, so
/// each snapshot pins down the linear scale's ticks and grid, the tick labels, the dataset
/// colors and the line path for one stepped mode.
#[test]
fn scatter_chart_stepped_modes() {
    let mut stepped = LocalState::new(Stepped::None);
    let mut harness = Harness::new(example_chart(stepped.clone()), Dimension::new(600.0, 500.0));

    for (mode, name) in [
        (Stepped::None, "stepped_none"),
        (Stepped::Before, "stepped_before"),
        (Stepped::After, "stepped_after"),
        (Stepped::Middle, "stepped_middle"),
        (Stepped::MiddleVertical, "stepped_middle_vertical"),
    ] {
        *stepped.value_mut() = mode;
        harness.snapshot(name);
    }
}

/// A plain scatter chart: two datasets of (x, y) points with numeric x values, points only, as
/// Chart.js's scatter chart draws them by default (`showLine: false`). Pins down that the
/// `ScatterController` draws no lines unless asked to, and keeps linear x and y scales.
#[test]
fn scatter_points_only() {
    let mut harness = Harness::new(scatter_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_points_only");
}

/// The point clouds of `scatter_points_only` with `show_line(true)`, the second one also
/// `stepped(After)`. Pins down that a scatter chart's lines connect the points in data order, not
/// sorted by x, so they double back like Chart.js's `showLine`; and that stepped lines work on a
/// scatter chart too.
#[test]
fn scatter_with_lines() {
    let mut harness = Harness::new(scatter_with_lines_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_with_lines");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Chart, DatasetExt, LineController, ScatterController, Stepped};
    use carbide_core::state::LocalState;
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    /// The chart from `carbide/examples/chart/chart.rs`, with the example's `data7` and `data8`.
    pub fn example_chart(stepped: LocalState<Stepped>) -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0].stepped(stepped);
        let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        Chart::new(LineController::new((data7, data8)))
            .framed()
    }

    /// Two clouds of (x, y) points with numeric x values, drawn as points only.
    pub fn scatter_chart() -> impl Widget {
        let first = vec![(-10.0, 0.0), (0.0, 10.0), (10.0, 5.0), (0.5, 5.5), (-4.0, 7.5), (6.0, -2.0), (3.0, 3.0)];
        let second = vec![(-8.0, -4.0), (-2.0, 2.0), (2.5, -6.0), (7.5, 8.0), (9.0, 1.5), (-6.5, 4.0)];

        Chart::new(ScatterController::new((first, second)))
            .framed()
    }

    /// `scatter_chart`'s points with lines through them in data order, the second dataset stepped.
    pub fn scatter_with_lines_chart() -> impl Widget {
        let first = vec![(-10.0, 0.0), (0.0, 10.0), (10.0, 5.0), (0.5, 5.5), (-4.0, 7.5), (6.0, -2.0), (3.0, 3.0)];
        let second = vec![(-8.0, -4.0), (-2.0, 2.0), (2.5, -6.0), (7.5, 8.0), (9.0, 1.5), (-6.5, 4.0)]
            .stepped(Stepped::After);

        Chart::new(ScatterController::new((first, second)).show_line(true))
            .framed()
    }
}
