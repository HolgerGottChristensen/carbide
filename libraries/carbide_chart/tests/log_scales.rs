//! Logarithmic scales: equal ratios at equal distances, ticks on the powers of the base with
//! minor ticks between them, and no place for zero or below.

use carbide_chart::{Axis, BarController, Chart, LineController, LogScale, ScaleId, ScatterController};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// A line growing from 2 to 40 000 over six steps, on a log y scale. Pins down that the range is
/// rounded out to whole powers (1 to 100 000), labelled ticks fall on each power and are evenly
/// spaced, and fainter grid lines and shorter tick marks without labels mark 2 to 9 times each
/// power.
#[test]
fn line_over_five_powers() {
    let growth = vec![2.0, 15.0, 120.0, 900.0, 7_000.0, 40_000.0];

    let chart = Chart::new(LineController::new(growth))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("line_over_five_powers");
}

/// Bars on a log y scale, one of them zero and one negative. Pins down that bars grow from the
/// scale's minimum, that the range starts at the smallest positive value rather than at the zero
/// every bar includes, and that the zero and the negative bar, which have no place, aren't drawn.
#[test]
fn bars_from_the_minimum() {
    let sales = vec![30.0, 0.0, 450.0, -20.0, 6_000.0];

    let chart = Chart::new(BarController::new(sales))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_from_the_minimum");
}

/// Scatter points with x from 0.05 to 800, on a log x scale, and a point at x = 0 that has no
/// place there. Pins down a horizontal log scale, with powers below one, minor ticks, and the
/// point at zero left out.
#[test]
fn x_scale_with_scatter_points() {
    let measurements = vec![(0.05, 1.0), (0.4, 2.5), (3.0, 3.0), (25.0, 4.5), (800.0, 6.0), (0.0, 3.5)];

    let chart = Chart::new(ScatterController::new(measurements))
        .scale(ScaleId::X, LogScale::new(Axis::Horizontal))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("x_scale_with_scatter_points");
}

/// Values spanning eighteen powers of ten, from 0.001 to 10¹⁵. Pins down that with too many
/// powers for one tick each, every second power gets a tick, and that there are no minor ticks.
#[test]
fn many_powers() {
    let values = vec![0.001, 1.0, 1e5, 1e9, 1e15];

    let chart = Chart::new(LineController::new(values))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("many_powers");
}

/// Values from 3 to 50 on a base 2 scale. Pins down that the base places the ticks, on 2, 4, 8,
/// 16, 32 and 64, without minor ticks, as there is no whole multiple between powers of two.
#[test]
fn base_two() {
    let values = vec![3.0, 7.0, 12.0, 30.0, 50.0];

    let chart = Chart::new(LineController::new(values))
        .scale(ScaleId::Y, LogScale::new(Axis::Vertical).base(2.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("base_two");
}
