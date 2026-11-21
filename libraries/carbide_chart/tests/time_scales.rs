//! Time scales: dates as coordinates, with ticks on round times in the scale's time zone and
//! labels showing dates or times for the locale, or text of your own.

use carbide_chart::{Axis, BarController, Chart, LineController, ScaleId, ScatterController, TimeScale, TimeScaleZone};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;
use chrono::{DateTime, FixedOffset};

mod common;

use common::Framed;
use support::{at, utc};

/// A week of daily temperatures as a line, read at noon, on a time x scale in UTC. Pins down that
/// ticks fall on midnights, a day apart, with the range rounded out to whole days, that the
/// points sit half a day past their ticks, and that the labels are short dates in the locale.
#[test]
fn days_on_a_line() {
    let temperatures: Vec<(DateTime<FixedOffset>, f64)> = (0..7)
        .map(|day| (at(2026, 3, 2 + day, 12, 0), [8.0, 9.5, 7.0, 11.0, 12.5, 10.0, 13.0][day as usize]))
        .collect();

    let chart = Chart::new(LineController::new(temperatures))
        .scale(ScaleId::X, TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc())))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("days_on_a_line");
}

/// Readings through a working day as scatter points, at uneven times. Pins down that a range of
/// hours gets ticks on whole hours, an even number of hours apart, labelled with the time of day
/// in the locale.
#[test]
fn hours_on_scatter_points() {
    let readings = vec![
        (at(2026, 3, 4, 8, 10), 3.2),
        (at(2026, 3, 4, 9, 45), 4.1),
        (at(2026, 3, 4, 11, 5), 3.8),
        (at(2026, 3, 4, 13, 30), 5.6),
        (at(2026, 3, 4, 15, 20), 5.1),
        (at(2026, 3, 4, 17, 50), 6.4),
    ];

    let chart = Chart::new(ScatterController::new(readings))
        .scale(ScaleId::X, TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc())))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("hours_on_scatter_points");
}

/// Revenue for each month of a half year as bars, each on the first of its month. Pins down that
/// ticks fall on the first of each month, so they are a calendar month apart rather than evenly
/// spaced, that bars sit on their ticks, and that the scale is offset by half a step at both
/// ends, so the first and last bars aren't cut in half.
#[test]
fn months_with_bars() {
    let revenue: Vec<(DateTime<FixedOffset>, f64)> = (1..=6)
        .map(|month| (at(2026, month, 1, 0, 0), [42.0, 38.0, 51.0, 47.0, 55.0, 60.0][month as usize - 1]))
        .collect();

    let chart = Chart::new(BarController::new(revenue))
        .scale(ScaleId::X, TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc())))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("months_with_bars");
}

/// The same months, labelled with text of your own: the month's short name, and the year under
/// January. Pins down that `format` replaces the locale's dates, given each tick's date in the
/// scale's zone and the unit the ticks are spaced in.
#[test]
fn labels_of_your_own() {
    let revenue: Vec<(DateTime<FixedOffset>, f64)> = (1..=6)
        .map(|month| (at(2026, month, 1, 0, 0), [42.0, 38.0, 51.0, 47.0, 55.0, 60.0][month as usize - 1]))
        .collect();

    let months = TimeScale::new(Axis::Horizontal)
        .time_zone(TimeScaleZone::Fixed(utc()))
        .format(|date, _unit| date.format("%b").to_string());

    let chart = Chart::new(BarController::new(revenue)).scale(ScaleId::X, months).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("labels_of_your_own");
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
