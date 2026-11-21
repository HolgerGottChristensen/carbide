//! Area charts: lines filled to a baseline or down to the line below them in a stack, and bands
//! between each point's low and high value.

use carbide_chart::{Chart, DataColor, DatasetExt, Fill, Interpolation, LineController, Range, RangeDataSetExt, ScatterController, StackId, Stepped};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::band_chart;

/// The example chart with its first dataset as a monotone area, once per baseline: zero, the
/// bottom and the top of the chart area, and the value 50. Pins down that the area follows the
/// curve exactly and closes along its baseline; that it fills up to zero where the line is below
/// it; that it is translucent and drawn under both datasets' lines and points; and that the
/// value baseline is drawn at 50.
#[test]
fn fill_modes() {
    // The example's data with `data7` as a monotone line filled to `fill`.
    let mut fill = LocalState::new(Fill::Origin);
    let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0]
        .interpolation(Interpolation::Monotone)
        .fill(fill.clone());
    let data8 = vec![-5.0, -10.0, 13.0, 92.0, 10.0, -53.0, 33.0];

    let chart = Chart::new(LineController::new((data7, data8)))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

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

/// The example data's first dataset filled to zero, once stepped (middle) and once smooth
/// (tension 0.4). Pins down that an area's top follows the line exactly for these
/// interpolations too: along every step, and along the spline where it overshoots the points.
#[test]
fn stepped_and_smooth() {
    // The example's `data7` alone, with `interpolation`, filled to zero.
    let filled_line_chart = |interpolation: Interpolation| {
        let data7 = vec![75.0, 70.0, 80.0, -60.0, 75.0, 98.0, -100.0]
            .interpolation(interpolation)
            .fill(Fill::Origin);

        Chart::new(LineController::new(data7))
            .framed()
    };

    let mut harness = Harness::new(filled_line_chart(Interpolation::Stepped(Stepped::Middle)), Dimension::new(600.0, 500.0));
    harness.snapshot("stepped");

    let mut harness = Harness::new(filled_line_chart(Interpolation::Smooth { tension: 0.4 }), Dimension::new(600.0, 500.0));
    harness.snapshot("smooth");
}

/// Three monotone lines in one stack, filled as a stacked area chart. Pins down that each area
/// fills exactly between its line and the line below, following that line's curve back, and the
/// bottom one down to zero, with the middle dataset's dip below zero lowering the lines above it.
#[test]
fn stacked() {
    // Visits per channel, stacked into one total: search, social (which dips below zero, as
    // refunds outweigh it one month) and email, all monotone, filled down to the line below.
    const VISITS: StackId = StackId::new("visits");

    let search = vec![30.0, 35.0, 32.0, 40.0, 45.0, 42.0, 50.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);
    let social = vec![10.0, 15.0, -8.0, 12.0, 20.0, 18.0, 25.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);
    let email = vec![12.0, 8.0, 15.0, 10.0, 14.0, 20.0, 16.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);

    let chart = Chart::new(LineController::new((search, social, email)))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("stacked");
}

/// Two stacked areas, the lower one missing a value in the middle. Pins down that the lower line
/// and its area break at the gap; that the missing value adds nothing to the stack, so the upper
/// line dips to its own value there; and that the upper area stays whole, its lower edge joining
/// the lower line's points straight across the gap.
#[test]
fn stacked_over_a_gap() {
    // Two stacked areas; the lower one has no value in the fourth month.
    const VISITS: StackId = StackId::new("visits");

    let search = vec![Some(30.0), Some(35.0), Some(32.0), None, Some(45.0), Some(42.0), Some(50.0)]
        .fill(Fill::Stack)
        .stack(VISITS);
    let email = vec![12.0, 8.0, 15.0, 10.0, 14.0, 20.0, 16.0]
        .fill(Fill::Stack)
        .stack(VISITS);

    let chart = Chart::new(LineController::new((search, email)))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_over_a_gap");
}

/// A temperature range as one band: the daily minimum as a plain line, and the spread up to the
/// maximum stacked on it as a line filled with `Fill::Stack`. Pins down that a stacked area fills
/// down to the line below even when that line has no fill of its own, so the only filled area is
/// the band bounded by the two lines; and that the y scale covers the band, not zero, as nothing
/// fills down to it.
#[test]
fn between_stacked_lines() {
    let mut harness = Harness::new(band_chart(Interpolation::Linear, Interpolation::Linear), Dimension::new(600.0, 500.0));

    harness.snapshot("between_stacked_lines");
}

/// The band of `area_between_stacked_lines` with a monotone lower line and a stepped upper line.
/// Pins down that each edge of the band follows its own line as drawn: the top edge steps with
/// the upper line, and the bottom edge runs back along the lower line's curve, so the band fits
/// exactly between them.
#[test]
fn between_stacked_lines_of_different_interpolations() {
    let mut harness = Harness::new(
        band_chart(Interpolation::Monotone, Interpolation::Stepped(Stepped::Middle)),
        Dimension::new(600.0, 500.0),
    );

    harness.snapshot("between_stacked_lines_of_different_interpolations");
}

/// Three monotone lines in one stack, filled as a stacked area chart, laid down. Pins down that
/// each line is drawn at the running total along the x scale, and that each area fills exactly
/// between its line and the line before it in the stack, sideways.
#[test]
fn horizontal_stacked() {
    const VISITS: StackId = StackId::new("visits");

    let search = vec![30.0, 35.0, 32.0, 40.0, 45.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);
    let social = vec![10.0, 15.0, 8.0, 12.0, 20.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);
    let email = vec![12.0, 8.0, 15.0, 10.0, 14.0]
        .interpolation(Interpolation::Monotone).fill(Fill::Stack).stack(VISITS);

    let chart = Chart::new(LineController::new((search, social, email)).horizontal())
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_stacked");
}

/// A scatter chart with lines, filled to zero, its points in x order with numeric x values. Pins
/// down that a scatter chart draws the area when it draws lines, under the line and its points,
/// and that the y scale reaches down to zero for it.
#[test]
fn scatter_filled_to_zero() {
    // Readings at uneven x values, in x order, as a scatter chart with lines filled to zero.
    let readings = vec![(-10.0, 2.0), (-6.0, 7.0), (-1.0, 4.0), (3.0, 9.0), (7.0, 5.0), (10.0, 8.0)]
        .fill(Fill::Origin);

    let chart = Chart::new(ScatterController::new(readings).show_line(true))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("filled_scatter_chart");
}

/// A month's temperature range as a band, with the month's average as a line through it, in one
/// line controller. Pins down that the band fills between the line through the lows and the line
/// through the highs, translucent, with both edges stroked; that it is drawn under the other
/// dataset's line; and that the y scale fits the band without reaching down to zero.
#[test]
fn range_band() {
    let temperatures = vec![
        Range { x: 0.0, low: 2.0, high: 9.0 },
        Range { x: 1.0, low: 3.0, high: 11.0 },
        Range { x: 2.0, low: 6.0, high: 15.0 },
        Range { x: 3.0, low: 9.0, high: 19.0 },
        Range { x: 4.0, low: 12.0, high: 22.0 },
        Range { x: 5.0, low: 11.0, high: 20.0 },
    ];
    let average = vec![5.5, 7.0, 10.5, 14.0, 17.0, 15.5];

    let chart = Chart::new(LineController::new((temperatures.range(), average))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("range_band");
}

/// A monotone band with a missing high in the middle, beside a stepped band. Pins down that a
/// band's edges follow its dataset's interpolation, both edges alike, and that a missing low or
/// high breaks the band into two, as a missing value breaks a line.
#[test]
fn range_bands_with_curves_and_a_gap() {
    let monotone: Vec<Range<f64, Option<f64>>> = vec![
        Range { x: 0.0, low: Some(10.0), high: Some(14.0) },
        Range { x: 1.0, low: Some(12.0), high: Some(18.0) },
        Range { x: 2.0, low: Some(11.0), high: Some(16.0) },
        Range { x: 3.0, low: Some(13.0), high: None },
        Range { x: 4.0, low: Some(15.0), high: Some(21.0) },
        Range { x: 5.0, low: Some(14.0), high: Some(19.0) },
        Range { x: 6.0, low: Some(17.0), high: Some(23.0) },
    ];
    let stepped = vec![
        Range { x: 0.0, low: 2.0, high: 5.0 },
        Range { x: 1.0, low: 3.0, high: 7.0 },
        Range { x: 2.0, low: 5.0, high: 6.0 },
        Range { x: 3.0, low: 4.0, high: 9.0 },
        Range { x: 4.0, low: 6.0, high: 8.0 },
        Range { x: 5.0, low: 3.0, high: 6.0 },
        Range { x: 6.0, low: 5.0, high: 10.0 },
    ];

    let chart = Chart::new(LineController::new((
        monotone.range().interpolation(Interpolation::Monotone),
        stepped.range().interpolation(Interpolation::Stepped(Stepped::Middle)),
    ))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("range_bands_with_curves_and_a_gap");
}

/// A lying band with a lying line through it. Pins down that the band follows the controller's
/// orientation, the categories running down the y scale and the band spreading along the x scale.
#[test]
fn horizontal_range_band() {
    let durations = vec![
        Range { x: 0.0, low: 3.0, high: 8.0 },
        Range { x: 1.0, low: 5.0, high: 12.0 },
        Range { x: 2.0, low: 4.0, high: 9.0 },
        Range { x: 3.0, low: 7.0, high: 15.0 },
    ];
    let median = vec![5.0, 8.0, 6.5, 11.0];

    let chart = Chart::new(LineController::new((durations.range(), median)).horizontal()).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_range_band");
}

/// A straight band and a monotone band whose highs and lows cross twice, with an inverted colour.
/// Pins down that the parts where the line through the highs runs below the line through the lows
/// are filled in the inverted colour, split exactly where the lines cross, on straight and on
/// curved edges alike, while the edges keep the dataset's colour.
#[test]
fn inverted_range_bands() {
    let straight = vec![
        Range { x: 0.0, low: 10.0, high: 16.0 },
        Range { x: 1.0, low: 12.0, high: 15.0 },
        Range { x: 2.0, low: 14.0, high: 11.0 },
        Range { x: 3.0, low: 13.0, high: 9.0 },
        Range { x: 4.0, low: 11.0, high: 14.0 },
        Range { x: 5.0, low: 10.0, high: 17.0 },
    ];
    let curved = vec![
        Range { x: 0.0, low: 2.0, high: 7.0 },
        Range { x: 1.0, low: 4.0, high: 6.0 },
        Range { x: 2.0, low: 6.0, high: 3.0 },
        Range { x: 3.0, low: 5.0, high: 2.0 },
        Range { x: 4.0, low: 3.0, high: 6.0 },
        Range { x: 5.0, low: 2.0, high: 8.0 },
    ];

    let chart = Chart::new(LineController::new((
        straight.range().inverted_color(DataColor::Palette(3)),
        curved.range().inverted_color(DataColor::Palette(3)).interpolation(Interpolation::Monotone),
    ))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("inverted_range_bands");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Chart, DatasetExt, Fill, Interpolation, LineController, StackId};
    
    use carbide_core::widget::Widget;
    use crate::common::Framed;

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
}
