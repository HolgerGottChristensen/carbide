//! Line charts: lines through points, straight, smooth, monotone or stepped, breaking at
//! gaps, stacked, and lying down.

use carbide_chart::{Chart, DatasetExt, Fill, Interpolation, LineController, SpanGaps, StackId, Stepped};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::example_chart;

/// The example chart with its first dataset's line smooth: a tension spline (tension 0.4), then
/// monotone. The data swings hard (98 to -100 at the end), so the spline overshoots the points
/// between them, and is kept inside the chart area where it would leave it; the monotone line turns
/// exactly at each point and never goes past them.
#[test]
fn smooth_modes() {
    let mut interpolation = LocalState::new(Interpolation::Smooth { tension: 0.4 });
    let mut harness = Harness::new(example_chart(interpolation.clone()), Dimension::new(600.0, 500.0));
    harness.snapshot("smooth");

    *interpolation.value_mut() = Interpolation::Monotone;
    harness.snapshot("monotone");
}

/// Renders the chart from `carbide/examples/chart/chart.rs` (two datasets, the first one's
/// `interpolation` driven by a state, Danish locale, bordered) and snapshots it with straight lines
/// and once per `Stepped` variant. The datasets are fixed, so each snapshot pins down the linear
/// scale's ticks and grid, the tick labels, the dataset colors and the line path for one mode.
/// `stepped_none` is the straight line, from before `Stepped::None` became `Interpolation::Linear`.
#[test]
fn stepped_modes() {
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

/// A filled monotone line with missing values: one missing month, then two in a row. Drawn once
/// per `SpanGaps` mode. Pins down that with `Never` the line, its curve and its area break at
/// both gaps, leaving the points around them on their own; with `Always` they run across both, as
/// one curve through the points that are there; and with `Within(2.5)` the one-month gap (2 apart
/// along x) is joined and the two-month gap (3 apart) breaks.
#[test]
fn gaps() {
    // Monthly sales with one missing month, then two missing in a row, as a filled monotone line
    // that spans gaps as `span_gaps` says.
    let mut span_gaps = LocalState::new(SpanGaps::Never);
    let sales = vec![Some(12.0), Some(15.0), None, Some(10.0), Some(14.0), None, None, Some(9.0), Some(13.0), Some(11.0)]
        .interpolation(Interpolation::Monotone)
        .fill(Fill::Origin)
        .span_gaps(span_gaps.clone());

    let chart = Chart::new(LineController::new(sales))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    for (mode, name) in [
        (SpanGaps::Never, "never"),
        (SpanGaps::Always, "always"),
        (SpanGaps::Within(2.5), "within"),
    ] {
        *span_gaps.value_mut() = mode;
        harness.snapshot(name);
    }
}

/// Three monotone lines in one stack. Pins down that each line is drawn at the running total of
/// the datasets up to it, with one sum, so the middle dataset's dip below zero lowers the lines
/// above it, and that the y scale covers the totals.
#[test]
fn stacked() {
    // Visits per channel, stacked into one total: search, social (which dips below zero, as
    // refunds outweigh it one month) and email, all monotone.
    const VISITS: StackId = StackId::new("visits");

    let search = vec![30.0, 35.0, 32.0, 40.0, 45.0, 42.0, 50.0]
        .interpolation(Interpolation::Monotone).stack(VISITS);
    let social = vec![10.0, 15.0, -8.0, 12.0, 20.0, 18.0, 25.0]
        .interpolation(Interpolation::Monotone).stack(VISITS);
    let email = vec![12.0, 8.0, 15.0, 10.0, 14.0, 20.0, 16.0]
        .interpolation(Interpolation::Monotone).stack(VISITS);

    let chart = Chart::new(LineController::new((search, social, email)))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("stacked");
}

/// A monotone line filled to zero and a stepped line, both laid down. Pins down that what follows
/// the categories follows them down the chart: the monotone curve never overshoots between two
/// rows, the steps change value halfway between rows, and the area fills sideways, between the
/// line and zero on the x scale, left of zero where the values are negative.
#[test]
fn horizontal() {
    let margin = vec![8.0, 12.0, -4.0, 6.0, 14.0, 10.0]
        .interpolation(Interpolation::Monotone)
        .fill(Fill::Origin);
    let budget = vec![5.0, 5.0, 9.0, 9.0, 12.0, 12.0]
        .interpolation(Interpolation::Stepped(Stepped::Middle));

    let chart = Chart::new(LineController::new((margin, budget)).horizontal(true))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Chart, DatasetExt, Interpolation, LineController};
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
}
