//! Line and scatter charts: points, lines through them, straight, smooth or stepped, and the
//! areas under them.

use carbide_chart::{DataColor, DatasetExt, Fill, Interpolation, Stepped};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use support::{band_chart, example_chart, filled_values_chart, filled_chart, filled_line_chart, filled_scatter_chart, scatter_chart, scatter_with_lines_chart, stacked_lines_chart};

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

/// The example chart with its first dataset as a monotone area, once per baseline: zero, the
/// bottom and the top of the chart area, and the value 50. Pins down that the area follows the
/// curve exactly and closes along its baseline; that it fills up to zero where the line is below
/// it; that it is translucent and drawn under both datasets' lines and points; and that the
/// value baseline is drawn at 50.
#[test]
fn area_fill_modes() {
    let mut fill = LocalState::new(Fill::Origin);
    let mut harness = Harness::new(filled_chart(fill.clone()), Dimension::new(600.0, 500.0));

    for (mode, name) in [
        (Fill::Origin, "origin"),
        (Fill::Bottom, "bottom"),
        (Fill::Top, "top"),
        (Fill::Value(50.0), "value"),
    ] {
        *fill.value_mut() = mode;
        harness.snapshot(name);
    }
}

/// Three monotone lines in one stack, first as lines, then filled as a stacked area chart. Pins
/// down that each line is drawn at the running total of the datasets up to it, with one sum, so
/// the middle dataset's dip below zero lowers the lines above it; that each area fills exactly
/// between its line and the line below, following that line's curve back, and the bottom one
/// down to zero; and that the y scale covers the totals.
#[test]
fn stacked_lines_and_areas() {
    let mut fill = LocalState::new(Fill::None);
    let mut harness = Harness::new(stacked_lines_chart(fill.clone()), Dimension::new(600.0, 500.0));
    harness.snapshot("stacked_lines");

    *fill.value_mut() = Fill::Stack;
    harness.snapshot("stacked_areas");
}

/// The example data's first dataset filled to zero, once stepped (middle) and once smooth
/// (tension 0.4). Pins down that an area's top follows the line exactly for these
/// interpolations too: along every step, and along the spline where it overshoots the points.
#[test]
fn filled_stepped_and_smooth_lines() {
    let mut harness = Harness::new(filled_line_chart(Interpolation::Stepped(Stepped::Middle)), Dimension::new(600.0, 500.0));
    harness.snapshot("stepped");

    let mut harness = Harness::new(filled_line_chart(Interpolation::Smooth { tension: 0.4 }), Dimension::new(600.0, 500.0));
    harness.snapshot("smooth");
}

/// A filled line coloured `ByPoint`. Pins down that each point takes the palette colour for its
/// index, wrapping around, while the line and its area, drawn once for the whole dataset, keep
/// the dataset's own palette colour (white, as the first dataset).
#[test]
fn points_coloured_by_point() {
    let values = vec![12.0, 19.0, 8.0, 15.0, 10.0, 17.0]
        .color(DataColor::ByPoint)
        .fill(Fill::Origin);
    let mut harness = Harness::new(filled_values_chart(values), Dimension::new(600.0, 500.0));

    harness.snapshot("points_coloured_by_point");
}

/// A temperature range as one band: the daily minimum as a plain line, and the spread up to the
/// maximum stacked on it as a line filled with `Fill::Stack`. Pins down that a stacked area fills
/// down to the line below even when that line has no fill of its own, so the only filled area is
/// the band bounded by the two lines; and that the y scale covers the band, not zero, as nothing
/// fills down to it.
#[test]
fn area_between_stacked_lines() {
    let mut harness = Harness::new(band_chart(Interpolation::Linear, Interpolation::Linear), Dimension::new(600.0, 500.0));

    harness.snapshot("area_between_stacked_lines");
}

/// The band of `area_between_stacked_lines` with a monotone lower line and a stepped upper line.
/// Pins down that each edge of the band follows its own line as drawn: the top edge steps with
/// the upper line, and the bottom edge runs back along the lower line's curve, so the band fits
/// exactly between them.
#[test]
fn area_between_stacked_lines_of_different_interpolations() {
    let mut harness = Harness::new(
        band_chart(Interpolation::Monotone, Interpolation::Stepped(Stepped::Middle)),
        Dimension::new(600.0, 500.0),
    );

    harness.snapshot("area_between_stacked_lines_of_different_interpolations");
}

/// A scatter chart with lines, filled to zero, its points in x order with numeric x values. Pins
/// down that a scatter chart draws the area when it draws lines, under the line and its points,
/// and that the y scale reaches down to zero for it.
#[test]
fn scatter_filled_to_zero() {
    let mut harness = Harness::new(filled_scatter_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("filled_scatter_chart");
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
    use carbide_chart::{Chart, DataSet, DatasetExt, Fill, Interpolation, LineController, ScatterController, StackId, Stepped};
    use carbide_core::draw::Scalar;
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

    /// A line chart of a single dataset, as given.
    pub fn filled_values_chart<D: DataSet<Scalar, Scalar, Scalar> + Clone>(values: D) -> impl Widget {
        Chart::new(LineController::new(values))
            .framed()
    }

    /// The example's `data7` alone, with `interpolation`, filled to zero.
    pub fn filled_line_chart(interpolation: Interpolation) -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0]
            .interpolation(interpolation)
            .fill(Fill::Origin);

        Chart::new(LineController::new(data7))
            .framed()
    }

    /// The example's data with `data7` as a monotone line filled to `fill`.
    pub fn filled_chart(fill: LocalState<Fill>) -> impl Widget {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0]
            .interpolation(Interpolation::Monotone)
            .fill(fill);
        let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

        Chart::new(LineController::new((data7, data8)))
            .framed()
    }

    /// Visits per channel, stacked into one total: search, social (which dips below zero, as
    /// refunds outweigh it one month) and email, all monotone and filled to `fill`.
    pub fn stacked_lines_chart(fill: LocalState<Fill>) -> impl Widget {
        const VISITS: StackId = StackId::new("visits");

        let search = vec![30.0, 35.0, 32.0, 40.0, 45.0, 42.0, 50.0]
            .interpolation(Interpolation::Monotone).fill(fill.clone()).stack(VISITS);
        let social = vec![10.0, 15.0, -8.0, 12.0, 20.0, 18.0, 25.0]
            .interpolation(Interpolation::Monotone).fill(fill.clone()).stack(VISITS);
        let email = vec![12.0, 8.0, 15.0, 10.0, 14.0, 20.0, 16.0]
            .interpolation(Interpolation::Monotone).fill(fill).stack(VISITS);

        Chart::new(LineController::new((search, social, email)))
            .framed()
    }

    /// A week's temperatures as a band: the daily minimum as a line with `lower` interpolation,
    /// and the spread to the maximum stacked on it with `upper` interpolation, filled down to the
    /// minimum.
    pub fn band_chart(lower: Interpolation, upper: Interpolation) -> impl Widget {
        const RANGE: StackId = StackId::new("range");

        let minimum = vec![5.0, 7.0, 6.0, 9.0, 12.0, 11.0, 8.0]
            .interpolation(lower)
            .stack(RANGE);
        let spread = vec![6.0, 5.0, 8.0, 7.0, 6.0, 9.0, 7.0]
            .interpolation(upper)
            .fill(Fill::Stack)
            .stack(RANGE);

        Chart::new(LineController::new((minimum, spread)))
            .framed()
    }

    /// Readings at uneven x values, in x order, as a scatter chart with lines filled to zero.
    pub fn filled_scatter_chart() -> impl Widget {
        let readings = vec![(-10.0, 2.0), (-6.0, 7.0), (-1.0, 4.0), (3.0, 9.0), (7.0, 5.0), (10.0, 8.0)]
            .fill(Fill::Origin);

        Chart::new(ScatterController::new(readings).show_line(true))
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
