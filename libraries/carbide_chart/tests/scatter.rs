//! Scatter charts: points at numeric x and y values, with or without a line through them.

use carbide_chart::{Chart, DatasetExt, Interpolation, ScatterController, Stepped};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;

/// A plain scatter chart: two datasets of (x, y) points with numeric x values, points only, as a
/// scatter chart draws them by default. Pins down that the `ScatterController` draws no lines
/// unless asked to, and keeps linear x and y scales.
#[test]
fn points_only() {
    // Two clouds of (x, y) points with numeric x values, drawn as points only.
    let first = vec![(-10.0, 0.0), (0.0, 10.0), (10.0, 5.0), (0.5, 5.5), (-4.0, 7.5), (6.0, -2.0), (3.0, 3.0)];
    let second = vec![(-8.0, -4.0), (-2.0, 2.0), (2.5, -6.0), (7.5, 8.0), (9.0, 1.5), (-6.5, 4.0)];

    let chart = Chart::new(ScatterController::new((first, second)))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("points_only");
}

/// The point clouds of `scatter_points_only` with `show_line(true)`, the second one also
/// stepped (`Interpolation::Stepped(Stepped::After)`). Pins down that a scatter chart's lines
/// connect the points in data order, not sorted by x, so they double back; and that stepped
/// lines work on a scatter chart too.
#[test]
fn with_lines() {
    // `scatter_chart`'s points with lines through them in data order, the second dataset stepped.
    let first = vec![(-10.0, 0.0), (0.0, 10.0), (10.0, 5.0), (0.5, 5.5), (-4.0, 7.5), (6.0, -2.0), (3.0, 3.0)];
    let second = vec![(-8.0, -4.0), (-2.0, 2.0), (2.5, -6.0), (7.5, 8.0), (9.0, 1.5), (-6.5, 4.0)]
        .interpolation(Interpolation::Stepped(Stepped::After));

    let chart = Chart::new(ScatterController::new((first, second)).show_line(true))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("with_lines");
}
