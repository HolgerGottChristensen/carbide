//! Zoomed scales (`zoom` on a scale): the scale keeps fitting to all its data and shows a window of
//! it, with ticks for the part shown, and everything drawn on it clipped to the chart area.

use carbide_chart::{Axis, BarController, CategoryScale, Chart, DatasetExt, Fill, LinearScale, LineController, ScaleId, ScatterController, SizedDataSetExt, TimeScale, TimeScaleZone, ZoomWindow};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State};
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::{at, utc, wave};

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// A filled line of 60 points on a linear x scale, zoomed to the window from 0.3 to 0.55. Pins
/// down that x shows only that part of its range, with nice ticks for it; that the line and its
/// area run on past the chart area's left and right edges, clipped there, through the points
/// either side of the window; and that y keeps the range of all the data.
#[test]
fn a_zoomed_line() {
    let points: Vec<(f64, f64)> = (0..60).map(|i| (i as f64, wave(i))).collect();

    let chart = Chart::new(LineController::new(points.fill(Fill::Origin)))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(ZoomWindow::new(0.3, 0.55)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("a_zoomed_line");
}

/// The line of `a_zoomed_line`, its y scale fitting to the visible data. Pins down that y fits the
/// values inside x's window, and the line runs off the chart area's top and bottom where the
/// points just outside the window lie beyond them.
#[test]
fn fitting_the_visible_data() {
    let points: Vec<(f64, f64)> = (0..60).map(|i| (i as f64, wave(i))).collect();

    let chart = Chart::new(LineController::new(points))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(ZoomWindow::new(0.3, 0.55)))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).fit_visible(true))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("fitting_the_visible_data");
}

/// Grouped bars for twelve months, zoomed to a window that starts and ends partway through a
/// month. Pins down that the window snaps to whole months, from March to June, so no bar is cut;
/// that the bars grow wider to fill the space; and that the grid lines between months fall on the
/// chart area's edges.
#[test]
fn bars_snap_to_whole_categories() {
    let revenue = vec![12.0, 19.0, 3.0, 5.0, 2.0, 8.0, 14.0, 9.0, 11.0, 6.0, 15.0, 18.0];
    let costs = vec![8.0, 11.0, 6.0, 4.0, 3.0, 7.0, 9.0, 5.0, 10.0, 8.0, 12.0, 9.0];

    let chart = Chart::new(BarController::new((revenue, costs)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(MONTHS).zoom(ZoomWindow::new(0.18, 0.49)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_snap_to_whole_categories");
}

/// Horizontal bars zoomed along their category y scale, and vertical bars zoomed along their value
/// scale. Pins down that a vertical category scale's window counts from the first category, at
/// the top, so the window from 0 to 0.25 shows January to March; and that zooming the value scale
/// cuts the tall bars off at the chart area's top, the value ticks following the part shown.
#[test]
fn zoomed_along_either_scale() {
    let revenue = vec![12.0, 19.0, 3.0, 5.0, 2.0, 8.0, 14.0, 9.0, 11.0, 6.0, 15.0, 18.0];

    let chart = Chart::new(BarController::new(revenue.clone()).horizontal(true))
        .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(MONTHS).zoom(ZoomWindow::new(0.0, 0.25)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("horizontal_bars_zoomed_along_the_categories");

    let chart = Chart::new(BarController::new(revenue))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(MONTHS))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(ZoomWindow::new(0.0, 0.5)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("standing_bars_zoomed_along_the_values");
}

/// A line over a year on a time scale, zoomed to a fortnight. Pins down that the ticks shown,
/// and the unit they are labelled in, are picked for the part shown: days instead of months.
#[test]
fn a_zoomed_time_scale() {
    let readings: Vec<_> = (0..365).map(|day| (at(2026, 1, 1) + chrono::Duration::days(day), wave(day as usize))).collect();

    let chart = Chart::new(LineController::new(readings))
        .scale(ScaleId::X, TimeScale::new(Axis::Horizontal).time_zone(TimeScaleZone::Fixed(utc())).zoom(ZoomWindow::new(0.5, 0.54)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("a_zoomed_time_scale");
}

/// Bubbles zoomed along both scales. Pins down that both scales show their windows, and that
/// bubbles near the edges are cut by the chart area instead of drawn over the axes.
#[test]
fn bubbles_zoomed_along_both_scales() {
    let bubbles = vec![(1.0, 2.0, 10.0), (3.0, 6.0, 40.0), (4.5, 4.0, 25.0), (6.0, 7.5, 30.0), (8.0, 3.0, 15.0)];

    let chart = Chart::new(ScatterController::new(bubbles.bubbles()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(ZoomWindow::new(0.25, 0.75)))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(ZoomWindow::new(0.3, 0.9)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubbles_zoomed_along_both_scales");
}

/// A line whose x scale's zoom is a state, panned between the snapshots, then reset to the whole
/// range, and once set to a window that can't be shown. Pins down that the chart follows the
/// zoom set after it is built; that `ZoomWindow::WHOLE` shows the whole range again, as without a
/// zoom; and that a window reaching past the end is moved back inside, keeping its width.
#[test]
fn the_zoom_is_a_state() {
    let points: Vec<(f64, f64)> = (0..60).map(|i| (i as f64, wave(i))).collect();
    let mut zoom = LocalState::new(ZoomWindow::new(0.0, 0.25));

    let chart = Chart::new(LineController::new(points))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(zoom.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("first_quarter");

    *zoom.value_mut() = ZoomWindow::new(0.5, 0.75);
    harness.snapshot("panned_to_the_third_quarter");

    *zoom.value_mut() = ZoomWindow::new(0.9, 1.15);
    harness.snapshot("past_the_end_moved_back_to_the_last_quarter");

    *zoom.value_mut() = ZoomWindow::WHOLE;
    harness.snapshot("reset_to_the_whole_range");
}

// ===============================
//  Utility
// ===============================

mod support {
    use chrono::{DateTime, FixedOffset, TimeZone};

    pub fn utc() -> FixedOffset {
        FixedOffset::east_opt(0).unwrap()
    }

    pub fn at(year: i32, month: u32, day: u32) -> DateTime<FixedOffset> {
        utc().with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    /// A smooth wave with a slower swell, so zoomed parts of it differ.
    pub fn wave(i: usize) -> f64 {
        let x = i as f64;
        50.0 + 30.0 * (x / 5.0).sin() + 15.0 * (x / 17.0).cos()
    }
}
