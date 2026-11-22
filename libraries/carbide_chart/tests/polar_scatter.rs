//! Scatter charts around a circle: a scatter controller on two radial scales, its x scale the
//! angle and its y scale the radius. Covers what a scatter controller draws (points, lines through
//! them, fills, bubbles, colours, the legend) and the scales it can draw on (linear, log, category
//! and time angles, partial sweeps, holes).

use carbide_chart::{AngleDirection, Axis, CategoryScale, Chart, DataColor, DatasetExt, Estimate, Fill, Interpolation, LegendPosition, LineController, LinearScale, LogScale, RangeDataSetExt, ScaleId, ScatterController, SizedDataSetExt, SpanGaps, Stepped, TimeScale, TimeScaleZone};
use carbide_core::draw::{Angle, Dimension};
use carbide_testing::Harness;
use chrono::{DateTime, FixedOffset};

mod common;

use common::Framed;
use support::{at, utc};

/// Three datasets of points without lines, on a linear angle scale, with a legend. Pins down that
/// each dataset's points are drawn in its own palette colour where their angle and radius put
/// them, that no line joins them, that the rings are circles, as scatter controllers ask, and that
/// the legend shows a point for each dataset.
#[test]
fn several_datasets_without_lines() {
    let north = vec![(10.0, 6.0), (40.0, 8.5), (75.0, 5.0), (300.0, 7.0), (340.0, 9.0)].label("North");
    let south = vec![(120.0, 4.0), (150.0, 6.5), (180.0, 8.0), (215.0, 5.5), (240.0, 3.0)].label("South");
    let calm = vec![(30.0, 1.5), (110.0, 2.0), (200.0, 1.0), (280.0, 2.5)].label("Calm");

    let chart = Chart::new(ScatterController::new((north, south, calm)))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("several_datasets_without_lines");
}

/// Open lines through scattered points, filled. Pins down that a scatter line around the circle
/// stays open, as scatter data isn't a loop; that `Fill::Bottom` fills it down to the centre, back
/// along nothing but the two ends' spokes, and `Fill::Top` out to the rim, back along the rim's
/// arc; and that `Fill::Origin` on a radius running from below zero fills to the ring at zero.
#[test]
fn filled_lines() {
    let bottom = vec![(20.0, 5.0), (80.0, 7.0), (140.0, 6.0), (200.0, 8.0)].fill(Fill::Bottom);
    let top = vec![(230.0, 6.0), (270.0, 8.0), (310.0, 5.0), (340.0, 7.0)].fill(Fill::Top);

    let chart = Chart::new(ScatterController::new((bottom, top)).show_line(true))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("to_the_centre_and_the_rim");

    let around_zero = vec![(10.0, 4.0), (70.0, -2.0), (130.0, 6.0), (190.0, -3.0), (250.0, 5.0), (310.0, 1.0)].fill(Fill::Origin);

    let chart = Chart::new(ScatterController::new(around_zero).show_line(true))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("to_zero");
}

/// Open scatter lines with each interpolation. Pins down that a smooth line is a spline through
/// the screen positions with free ends; that a monotone line draws as a smooth one, as there is no
/// direction to be monotone in around a circle; and that a stepped line runs along the arc at the
/// radius it leaves from, then along the next point's spoke, without a step back to the start.
#[test]
fn interpolated_lines() {
    let points = |offset: f64| vec![(offset, 4.0), (offset + 40.0, 7.0), (offset + 80.0, 5.0), (offset + 120.0, 8.0)];

    let smooth = points(0.0).interpolation(Interpolation::Smooth { tension: 0.4 }).label("Smooth");
    let monotone = points(120.0).interpolation(Interpolation::Monotone).label("Monotone");
    let stepped = points(240.0).interpolation(Interpolation::Stepped(Stepped::After)).label("Stepped");

    let chart = Chart::new(ScatterController::new((smooth, monotone, stepped)).show_line(true))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("interpolated_lines");
}

/// Scatter lines with missing values. Pins down that a missing value leaves its point out and
/// breaks the line by default; that `SpanGaps::Always` joins across it; and that
/// `SpanGaps::Within` joins across a gap narrower than its distance along the angle scale and
/// breaks a wider one.
#[test]
fn gaps() {
    let never = vec![(0.0, Some(8.0)), (30.0, Some(9.0)), (60.0, None), (90.0, Some(8.5)), (120.0, Some(9.5))].label("Never");
    let always = vec![(150.0, Some(5.0)), (180.0, None), (210.0, Some(6.0)), (240.0, Some(5.5))].span_gaps(SpanGaps::Always).label("Always");
    let within = vec![(250.0, 2.0), (270.0, 3.0), (290.0, 2.5), (350.0, 3.5)].span_gaps(SpanGaps::Within(30.0)).label("Within 30");

    let chart = Chart::new(ScatterController::new((never, always, within)).show_line(true))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("gaps");
}

/// Points coloured by point, next to a dataset in a fixed palette colour, with a controller
/// palette offset. Pins down that `DataColor::ByPoint` gives each point the next palette colour by
/// its index, starting at the controller's offset, and that a dataset picking a palette colour
/// isn't shifted.
#[test]
fn coloured_by_point() {
    let by_point = vec![(15.0, 5.0), (75.0, 7.0), (135.0, 6.0), (195.0, 8.0), (255.0, 4.0), (315.0, 6.5)].color(DataColor::ByPoint);
    let picked = vec![(45.0, 2.0), (165.0, 3.0), (285.0, 2.5)].color(DataColor::Palette(0));

    let chart = Chart::new(ScatterController::new((by_point, picked)).palette_offset(2))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("coloured_by_point");
}

/// Bubbles of different sizes beside plain points, with a wider radius range, one bubble on the
/// rim. Pins down that bubbles compare on the controller's one size scale, the largest at the
/// largest radius set with `radius_range`; that plain points keep their size beside them; that a
/// bubble on the rim reaches over it, as a bubble on an edge of a cartesian chart does; and that
/// the legend shows a bubble for the bubbles and a point for the points.
#[test]
fn bubbles_beside_points() {
    let bubbles = vec![(20.0, 4.0, 2.0), (100.0, 6.0, 8.0), (190.0, 10.0, 20.0), (280.0, 5.0, 12.0)].bubbles().label("Storms");
    let points = vec![(60.0, 3.0), (150.0, 7.0), (230.0, 2.0), (330.0, 8.0)].label("Readings");

    let chart = Chart::new(ScatterController::new((bubbles, points)).radius_range(6.0, 40.0))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bubbles_beside_points");
}

/// Points with a line on three quarters of a circle, from the left round counter-clockwise past
/// the bottom to the top, with a hole in the middle and a log radius. Pins down that a linear
/// angle scale takes the start angle, end angle and direction as a category scale does; that a
/// partial sweep is fitted by its bounding box; that the radius starts at the hole; and that a log
/// radius places its values on the powers of ten, with fainter minor rings.
#[test]
fn partial_counter_clockwise_sweep_with_a_hole_and_a_log_radius() {
    let counts = vec![(0.0, 3.0), (20.0, 40.0), (45.0, 700.0), (70.0, 90.0), (100.0, 8_000.0)];

    let chart = Chart::new(ScatterController::new(counts).show_line(true))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial)
            .start_angle(Angle::Degrees(270.0))
            .end_angle(Angle::Degrees(540.0))
            .direction(AngleDirection::CounterClockwise))
        .scale(ScaleId::Y, LogScale::new(Axis::Radial).inner_radius(0.25))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("partial_counter_clockwise_sweep_with_a_hole_and_a_log_radius");
}

/// Angle scales that aren't numbers: weekdays named by the data, and the hours of a day on a time
/// scale, a clock. Pins down that text x values make categories around the circle, in the order
/// the data names them, the last one step before the first; and that times around a time scale
/// get ticks on round hours, labelled as times of day.
#[test]
fn categories_and_times_around_the_circle() {
    let first = vec![("Mon", 3.0), ("Tue", 5.0), ("Wed", 2.0), ("Thu", 6.0)];
    let second = vec![("Tue", 1.5), ("Fri", 4.0), ("Sat", 2.5), ("Sun", 3.5)];

    let chart = Chart::new(ScatterController::new((first, second)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("weekdays");

    let arrivals: Vec<(DateTime<FixedOffset>, f64)> = vec![
        (at(2026, 3, 4, 0, 40), 2.0),
        (at(2026, 3, 4, 5, 15), 1.0),
        (at(2026, 3, 4, 8, 30), 9.0),
        (at(2026, 3, 4, 12, 10), 6.0),
        (at(2026, 3, 4, 17, 45), 8.0),
        (at(2026, 3, 4, 21, 20), 4.0),
    ];

    let chart = Chart::new(ScatterController::new(arrivals).show_line(true))
        .scale(ScaleId::X, TimeScale::new(Axis::Radial).time_zone(TimeScaleZone::Fixed(utc())))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("hours_of_a_day");
}

/// A radar with scattered points on the same scales, the points named by category, and a
/// dataset with error bars. Pins down that both controllers draw around one circle, the points
/// at their categories' angles; that the scatter controller's circles win over the radar's
/// polygons for the rings; and that the dataset with error bars has its points drawn but no
/// whiskers, as overlays aren't drawn around a circle yet.
#[test]
fn with_a_radar_and_error_bars() {
    let skills = ["Speed", "Strength", "Stamina", "Passing", "Shooting", "Defence"];

    let radar = LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].fill(Fill::Bottom));
    let picks = vec![("Speed", 6.5), ("Stamina", 8.5), ("Shooting", 7.0)];
    let estimates = vec![
        Estimate { x: "Strength", y: 4.5, low: 3.5, high: 5.5 },
        Estimate { x: "Passing", y: 6.0, low: 5.0, high: 7.5 },
        Estimate { x: "Defence", y: 7.5, low: 6.5, high: 8.0 },
    ].error_bars();

    let chart = Chart::new((radar, ScatterController::new((picks, estimates))))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(skills))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("with_a_radar_and_error_bars");
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
