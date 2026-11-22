//! Radars: a line controller on two radial scales, its x scale the angle and its y scale the
//! radius. Covers what `polar.rs` doesn't: fills to a value's ring, the other interpolations and
//! gaps, colours, the legend, categories and times around the circle, marks that aren't drawn
//! around a circle yet, and a lying line's fill.

use carbide_chart::{Axis, CategoryScale, Chart, DataColor, DatasetExt, Estimate, Fill, GridShape, Interpolation, LinearScale, LegendPosition, LineController, Range, RangeDataSetExt, ScaleId, SpanGaps, Stepped, TimeScale, TimeScaleZone};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;
use chrono::{DateTime, FixedOffset};

mod common;

use common::Framed;
use support::{at, utc};

const SKILLS: [&str; 6] = ["Speed", "Strength", "Stamina", "Passing", "Shooting", "Defence"];

/// Radars filled to a value's ring on a radius running from below zero: one to zero, one to 4.
/// Pins down that `Fill::Origin` and `Fill::Value` fill between the loop and the ring at their
/// value, inside it where the loop dips below and outside where it rises above, leaving the disc
/// inside both unfilled; and that both values are included in the radius, so their rings are on
/// the chart.
#[test]
fn filled_to_a_value() {
    let to_zero = vec![4.0, -2.0, 6.0, -3.0, 5.0, 1.0].fill(Fill::Origin).label("To zero");
    let to_four = vec![7.0, 6.0, 8.0, 5.0, 7.5, 6.5].fill(Fill::Value(4.0)).label("To 4");

    let chart = Chart::new(LineController::new((to_zero, to_four)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("filled_to_a_value");
}

/// Radars with the interpolations `polar.rs` doesn't draw: monotone, and steps that turn before
/// and in the middle. Pins down that a monotone loop draws as a smooth one, closed without a
/// corner; that `Stepped::Before` runs out along the spoke first, then around the polygon ring at
/// the next point's radius, straight to the next corner; that `Stepped::Middle` turns on the
/// polygon's edge halfway between two corners; and that each closes with a step from the last
/// point back to the first.
#[test]
fn monotone_and_other_steps() {
    let monotone = vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].interpolation(Interpolation::Monotone).label("Monotone");
    let before = vec![5.0, 3.0, 4.0, 2.0, 3.5, 4.5].interpolation(Interpolation::Stepped(Stepped::Before)).label("Before");
    let middle = vec![2.0, 1.0, 2.5, 1.5, 1.0, 2.0].interpolation(Interpolation::Stepped(Stepped::Middle)).label("Middle");

    let chart = Chart::new(LineController::new((monotone, before, middle)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("monotone_and_other_steps");
}

/// The same steps as `monotone_and_other_steps`, on a radius scale whose rings are circles.
/// Pins down that steps follow the grid: around circles they turn along arcs, where around the
/// polygons a radar has by default they run straight from corner to corner.
#[test]
fn steps_around_circles() {
    let after = vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].interpolation(Interpolation::Stepped(Stepped::After)).label("After");
    let before = vec![5.0, 3.0, 4.0, 2.0, 3.5, 4.5].interpolation(Interpolation::Stepped(Stepped::Before)).label("Before");
    let middle = vec![2.0, 1.0, 2.5, 1.5, 1.0, 2.0].interpolation(Interpolation::Stepped(Stepped::Middle)).label("Middle");

    let chart = Chart::new(LineController::new((after, before, middle)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).grid_shape(GridShape::Circle))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("steps_around_circles");
}

/// Radars whose gaps `SpanGaps::Within` joins or breaks, by distance along the category scale.
/// Pins down that a single missing category, two categories apart, is joined within 2 and closes
/// the loop; and that two missing in a row, three apart, break the line within 2, so it stays
/// open.
#[test]
fn gaps_within_a_distance() {
    let one_missing = vec![Some(8.0), Some(7.0), None, Some(9.0), Some(8.5), Some(7.5)].span_gaps(SpanGaps::Within(2.0)).label("One missing");
    let two_missing = vec![Some(4.0), None, None, Some(5.0), Some(3.5), Some(4.5)].span_gaps(SpanGaps::Within(2.0)).label("Two missing");

    let chart = Chart::new(LineController::new((one_missing, two_missing)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("gaps_within_a_distance");
}

/// A radar coloured by point, next to one in a picked palette colour, in a controller with a
/// palette offset. Pins down that a dataset coloured `ByPoint` draws its loop in its own palette
/// colour and each point in the next palette colour by its index, starting at the offset, and
/// that a picked palette colour isn't shifted.
#[test]
fn coloured_by_point() {
    let by_point = vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].color(DataColor::ByPoint);
    let picked = vec![3.0, 4.0, 2.5, 3.5, 4.5, 3.0].color(DataColor::Palette(0));

    let chart = Chart::new(LineController::new((by_point, picked)).palette_offset(2))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("coloured_by_point");
}

/// Radars with and without fills, named, with the legend on the right. Pins down that a
/// radar's legend gives a line for a line and a square for a filled area, as for cartesian lines,
/// and that the circle is fitted into what the legend leaves.
#[test]
fn legend_with_filled_and_unfilled_lines() {
    let line = vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].label("Target");
    let filled = vec![6.0, 7.0, 5.0, 6.5, 7.5, 6.0].fill(Fill::Bottom).label("Actual");

    let chart = Chart::new(LineController::new((line, filled)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .legend(LegendPosition::Right)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("legend_with_filled_and_unfilled_lines");
}

/// Two radars naming their categories in their data, with no labels on the scale. Pins down that
/// the categories go around the circle in the order the data names them, across both datasets,
/// and that a dataset lacking a category has no point there: the second skips Wednesday, so its
/// loop runs straight from Tuesday to Thursday.
#[test]
fn categories_named_by_the_data() {
    let first = vec![("Mon", 5.0), ("Tue", 7.0), ("Wed", 4.0), ("Thu", 6.0), ("Fri", 8.0)];
    let second = vec![("Mon", 3.0), ("Tue", 4.0), ("Thu", 5.0), ("Fri", 3.5), ("Sat", 6.0)];

    let chart = Chart::new(LineController::new((first, second)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("categories_named_by_the_data");
}

/// A day's hourly load as a radar around a time scale, a clock. Pins down that times around the
/// circle get ticks on round hours, labelled as times of day; that the range is rounded out to
/// whole steps, so the day ends where it began; and that the line, though on a full circle,
/// closes from the last hour back to the first.
#[test]
fn around_a_clock() {
    let load: Vec<(DateTime<FixedOffset>, f64)> = (0..24)
        .map(|hour| (at(2026, 3, 4, hour, 0), [3.0, 2.5, 2.0, 2.0, 2.5, 3.5, 5.0, 7.0, 8.0, 7.5, 7.0, 7.5, 8.0, 7.5, 7.0, 7.0, 7.5, 8.5, 9.0, 8.0, 6.5, 5.0, 4.0, 3.5][hour as usize]))
        .collect();

    let chart = Chart::new(LineController::new(load.fill(Fill::Bottom)))
        .scale(ScaleId::X, TimeScale::new(Axis::Radial).time_zone(TimeScaleZone::Fixed(utc())))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("around_a_clock");
}

/// A radar with a band and one with error bars, next to a plain one. Pins down that a band on a
/// radar is the ring between the loop through its lows and the loop through its highs, both
/// closed and stroked in the dataset's colour, in place of the dataset's line; that error bars
/// run along each point's spoke, from its low to its high, with caps across the spoke, over every
/// dataset's lines; and that the plain radar is drawn as usual.
#[test]
fn band_and_error_bars() {
    let band = vec![
        Range { x: 0.0, low: 6.0, high: 9.0 },
        Range { x: 1.0, low: 5.0, high: 7.0 },
        Range { x: 2.0, low: 6.0, high: 8.0 },
        Range { x: 3.0, low: 7.0, high: 10.0 },
        Range { x: 4.0, low: 4.0, high: 6.0 },
        Range { x: 5.0, low: 3.0, high: 5.0 },
    ].range();
    let estimates = vec![
        Estimate { x: 0.0, y: 4.0, low: 3.0, high: 5.0 },
        Estimate { x: 1.0, y: 5.0, low: 4.0, high: 6.0 },
        Estimate { x: 2.0, y: 3.5, low: 2.5, high: 4.5 },
        Estimate { x: 3.0, y: 4.5, low: 3.5, high: 5.5 },
        Estimate { x: 4.0, y: 5.5, low: 4.5, high: 6.5 },
        Estimate { x: 5.0, y: 4.0, low: 3.0, high: 5.0 },
    ].error_bars();
    let plain = vec![2.0, 2.5, 1.5, 2.0, 3.0, 2.5];

    let chart = Chart::new(LineController::new((band, estimates, plain)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("band_and_error_bars");
}

/// A stepped band with a missing value, and error bars on a lying radar. Pins down that a band
/// broken by a gap is open, each run filled between its lows and highs back along the lows, its
/// steps turning around the polygon rings; and that on a lying radar, whose values run around the
/// circle, the whiskers run around the ring at each category's distance, from the low's angle to
/// the high's, with caps along the spoke.
#[test]
fn stepped_band_with_a_gap_and_error_bars_around_the_ring() {
    let band = vec![
        Range { x: 0.0, low: Some(6.0), high: Some(9.0) },
        Range { x: 1.0, low: Some(5.0), high: Some(7.0) },
        Range { x: 2.0, low: None, high: None },
        Range { x: 3.0, low: Some(7.0), high: Some(10.0) },
        Range { x: 4.0, low: Some(4.0), high: Some(6.0) },
        Range { x: 5.0, low: Some(3.0), high: Some(5.0) },
    ].range().interpolation(Interpolation::Stepped(Stepped::After));

    let chart = Chart::new(LineController::new(band))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("stepped_band_with_a_gap");

    let estimates = vec![
        Estimate { x: 0.0, y: 6.0, low: 5.0, high: 7.0 },
        Estimate { x: 1.0, y: 7.5, low: 6.5, high: 8.0 },
        Estimate { x: 2.0, y: 5.5, low: 4.5, high: 6.5 },
        Estimate { x: 3.0, y: 8.0, low: 7.0, high: 9.0 },
        Estimate { x: 4.0, y: 6.5, low: 5.5, high: 7.5 },
        Estimate { x: 5.0, y: 7.0, low: 6.0, high: 8.0 },
    ].error_bars();

    let chart = Chart::new(LineController::new(estimates).horizontal())
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("error_bars_around_the_ring");
}

/// A lying radar, its categories out along the radius, filled. Pins down that a line whose
/// categories run along the radius isn't filled, as there is no ring for its baseline, while its
/// line and points are drawn as without the fill.
#[test]
fn lying_line_with_a_fill() {
    let ratings = vec![7.3, 6.1, 6.8, 8.6, 5.2, 4.4].fill(Fill::Bottom);

    let chart = Chart::new(LineController::new(ratings).horizontal())
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("lying_line_with_a_fill");
}

// ===============================
//  Utility
// ===============================

mod support {
    use chrono::{DateTime, FixedOffset, TimeZone};

    /// UTC as a fixed offset, so the snapshots don't depend on the machine's time zone.
    pub fn utc() -> FixedOffset {
        FixedOffset::east_opt(0).unwrap()
    }

    /// The moment at the given date and time in UTC.
    pub fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<FixedOffset> {
        utc().with_ymd_and_hms(year, month, day, hour, minute, 0).unwrap()
    }
}
