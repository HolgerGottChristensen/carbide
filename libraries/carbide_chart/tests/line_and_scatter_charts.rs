//! Line and scatter charts: points, and lines through them, straight, smooth or stepped.

use carbide_chart::{Interpolation, Stepped};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use support::{example_chart, scatter_chart, scatter_with_lines_chart};

/// Renders the chart from `carbide/examples/chart/chart.rs` (two datasets, the first one's
/// `interpolation` driven by a state, Danish locale, bordered) and snapshots it with straight lines
/// and once per `Stepped` variant. The datasets are fixed, so each snapshot pins down the linear
/// scale's ticks and grid, the tick labels, the dataset colors and the line path for one mode.
/// `stepped_none` is the straight line, from before `Stepped::None` became `Interpolation::Linear`.
#[test]
fn scatter_chart_stepped_modes() {
    let mut interpolation = LocalState::new(Interpolation::Linear);
    let mut harness = Harness::new(example_chart(interpolation.clone()), Dimension::new(600.0, 500.0));

    for (mode, name) in [
        (Interpolation::Linear, "stepped_none"),
        (Interpolation::Stepped(Stepped::Before), "stepped_before"),
        (Interpolation::Stepped(Stepped::After), "stepped_after"),
        (Interpolation::Stepped(Stepped::Middle), "stepped_middle"),
        (Interpolation::Stepped(Stepped::MiddleVertical), "stepped_middle_vertical"),
    ] {
        *interpolation.value_mut() = mode;
        harness.snapshot(name);
    }
}

/// The example chart with its first dataset's line smooth: a tension spline (tension 0.4), then
/// monotone. The data swings hard (98 to -100 at the end), so the spline overshoots the points
/// between them, and is kept inside the chart area where it would leave it; the monotone line turns
/// exactly at each point and never goes past them.
#[test]
fn smooth_line_modes() {
    let mut interpolation = LocalState::new(Interpolation::Smooth { tension: 0.4 });
    let mut harness = Harness::new(example_chart(interpolation.clone()), Dimension::new(600.0, 500.0));
    harness.snapshot("smooth");

    *interpolation.value_mut() = Interpolation::Monotone;
    harness.snapshot("monotone");
}

/// A plain scatter chart: two datasets of (x, y) points with numeric x values, points only, as a
/// scatter chart draws them by default. Pins down that the `ScatterController` draws no lines
/// unless asked to, and keeps linear x and y scales.
#[test]
fn scatter_points_only() {
    let mut harness = Harness::new(scatter_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_points_only");
}

/// The point clouds of `scatter_points_only` with `show_line(true)`, the second one also
/// stepped (`Interpolation::Stepped(Stepped::After)`). Pins down that a scatter chart's lines
/// connect the points in data order, not sorted by x, so they double back; and that stepped
/// lines work on a scatter chart too.
#[test]
fn scatter_with_lines() {
    let mut harness = Harness::new(scatter_with_lines_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_with_lines");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Chart, DatasetExt, Interpolation, LineController, ScatterController, Stepped};
    use carbide_core::state::LocalState;
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    /// The chart from `carbide/examples/chart/chart.rs`, with the example's `data7` and `data8`.
    pub fn example_chart(interpolation: LocalState<Interpolation>) -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0].interpolation(interpolation);
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
            .interpolation(Interpolation::Stepped(Stepped::After));

        Chart::new(ScatterController::new((first, second)).show_line(true))
            .framed()
    }
}
