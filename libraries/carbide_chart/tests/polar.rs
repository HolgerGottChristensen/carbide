//! Polar charts: datasets on two radial scales, drawn around a circle, their x scale the angle and
//! their y scale the radius, or the other way round when their controller is `horizontal`. A line
//! controller on them draws a radar, a scatter controller scattered points; the scales set where
//! the circle starts and ends, which way it runs, the hole in the middle and the rings' shape.

use carbide_chart::{AngleDirection, Axis, BarController, CategoryScale, Chart, DatasetExt, Fill, GridShape, Interpolation, LegendPosition, LineController, LinearScale, LogScale, ScaleId, ScatterController, SizedDataSetExt, StackId, Stepped};
use carbide_core::draw::{Angle, Dimension};
use carbide_testing::Harness;

mod common;

use common::Framed;

/// What a player is rated on, around the radar.
const SKILLS: [&str; 6] = ["Speed", "Strength", "Stamina", "Passing", "Shooting", "Defence"];

/// Two players' ratings on a radar: a line controller on a radial category x scale, which is the
/// angle, and whose default y scale becomes radial too, the radius. Pins down that the categories start at the top and run clockwise,
/// a sixth of a turn apart, the last one before the first; that each line is a closed loop of
/// chords; that the rings are polygons through the spokes, as line controllers ask; that the
/// radius is fitted to the data, so the centre is the smallest tick, not zero; and that the labels
/// sit around the rim, the circle shrunk to make room for them.
#[test]
fn radar() {
    let first = vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].label("Ana");
    let second = vec![5.0, 9.0, 8.0, 4.0, 7.0, 8.0].label("Ben");

    let chart = Chart::new(LineController::new((first, second)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("radar");
}

/// A radar filled to the centre, and one filled out to the rim. Pins down that `Fill::Bottom`
/// fills the loop and `Fill::Top` the ring between the loop and the rim.
#[test]
fn filled_to_the_centre_and_the_rim() {
    let inside = vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].fill(Fill::Bottom);
    let outside = vec![3.0, 4.0, 2.0, 3.0, 4.0, 3.0].fill(Fill::Top);

    let chart = Chart::new(LineController::new((inside, outside)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("filled_to_the_centre_and_the_rim");
}

/// Three stacked areas around the circle. Pins down that stacked loops sit at their running
/// totals and that each fills the ring between itself and the loop below.
#[test]
fn stacked_areas() {
    const HOURS: StackId = StackId::new("hours");
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"];

    let lines = (
        vec![3.0, 4.0, 3.0, 5.0, 4.0, 3.0, 4.0, 5.0].fill(Fill::Bottom).stack(HOURS),
        vec![2.0, 2.0, 3.0, 2.0, 3.0, 4.0, 2.0, 2.0].fill(Fill::Stack).stack(HOURS),
        vec![1.0, 2.0, 1.0, 2.0, 1.0, 1.0, 3.0, 2.0].fill(Fill::Stack).stack(HOURS),
    );

    let chart = Chart::new(LineController::new(lines))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(months))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_areas");
}

/// A smooth radar and a stepped one. Pins down that a smooth loop is a closed curve with no corner
/// where it closes, and that steps run around the polygon ring at the radius they leave from,
/// straight to the next corner, then out or in along the next spoke, the last step back round to
/// the first point.
#[test]
fn smooth_and_stepped() {
    let smooth = vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].interpolation(Interpolation::Smooth { tension: 0.4 });
    let stepped = vec![4.0, 3.0, 5.0, 2.0, 3.0, 6.0].interpolation(Interpolation::Stepped(Stepped::After));

    let chart = Chart::new(LineController::new((smooth, stepped)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("smooth_and_stepped");
}

/// A radar whose radius scale asks for circles, running counter-clockwise from the right. Pins
/// down `grid_shape` overriding the line controller's polygons, `start_angle` turning where the
/// first category sits, and `direction` running the categories the other way round.
#[test]
fn circles_counter_clockwise_from_the_right() {
    let ratings = vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0];

    let chart = Chart::new(LineController::new(ratings))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS)
            .start_angle(Angle::Degrees(90.0))
            .end_angle(Angle::Degrees(450.0))
            .direction(AngleDirection::CounterClockwise))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).grid_shape(GridShape::Circle))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("circles_counter_clockwise_from_the_right");
}

/// A half circle across the top with a hole in the middle, its line filled to the hole. Pins down
/// that a partial sweep is fitted by its bounding box, so it uses the whole width; that its first
/// and last categories sit on its ends; that its line stays open, filled down to the inner arc;
/// and that the border closes the sector with the inner arc and both end spokes.
#[test]
fn half_circle_with_a_hole() {
    let load = vec![20.0, 45.0, 60.0, 80.0, 55.0, 30.0].fill(Fill::Bottom);
    let hours = ["00", "04", "08", "12", "16", "20"];

    let chart = Chart::new(LineController::new(load))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(hours)
            .start_angle(Angle::Degrees(-90.0))
            .end_angle(Angle::Degrees(90.0)))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).inner_radius(0.3))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("half_circle_with_a_hole");
}

/// A radar of values from 2 to 5 000 on a log radius scale. Pins down that any scale kind works
/// around the circle: rings on the powers of ten, fainter rings at the minor ticks.
#[test]
fn log_radius() {
    let counts = vec![2.0, 40.0, 900.0, 5_000.0, 300.0, 15.0];

    let chart = Chart::new(LineController::new(counts))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .scale(ScaleId::Y, LogScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("log_radius");
}

/// Scattered points and bubbles at numeric angles, with a line through the points. Pins down a
/// linear angle scale fitted to its data like any linear scale, from 0 to 350 here, whose end
/// lands on its start and is labelled once; a scatter controller's circle rings; an open line, as
/// scatter data isn't a loop; and bubbles drawn where their points are.
#[test]
fn scatter_and_bubbles() {
    let wind = vec![(0.0, 4.0), (45.0, 7.0), (100.0, 5.0), (170.0, 9.0), (250.0, 6.0), (320.0, 8.0)];
    let gusts = vec![(30.0, 3.0, 2.0), (140.0, 6.0, 5.0), (290.0, 2.0, 9.0)].bubbles();

    let chart = Chart::new(ScatterController::new((wind, gusts)).show_line(true))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_and_bubbles");
}

/// A radar over bars, in one chart: the bars on the default x and y scales, the radar on two
/// radial scales of its own. Pins down that the cartesian scales keep their bands, the
/// circle is fitted inside the chart area that is left, and controllers draw in order, the radar
/// over the bars.
#[test]
fn radar_over_bars() {
    let angle = ScaleId::new("angle");
    let radius = ScaleId::new("radius");

    let bars = BarController::new(vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0]);
    let radar = LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].x_scale(angle.clone()).y_scale(radius.clone()).fill(Fill::Bottom));

    let chart = Chart::new((bars, radar))
        .scale(angle, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .scale(radius, LinearScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("radar_over_bars");
}

/// Ratings with a `horizontal` line controller: the roles swap, not the data, so the categories
/// stay on the x scale, which is now the radius, and the values run around the circle on the y
/// scale, fitted from 4 to 9. Pins down that the labels set on the x scale stay where they are
/// set; that the categories sit along the start spoke, labelled beside it, the first at the centre
/// and the last at the rim, kept next to the rim label there; that the rings are polygons with a
/// corner at each of the y scale's ticks; and that the line, whose categories aren't around the
/// circle, stays open.
#[test]
fn categories_along_the_radius() {
    let ratings = vec![7.3, 6.1, 6.8, 8.6, 5.2, 4.4];

    let chart = Chart::new(LineController::new(ratings).horizontal(true))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("categories_along_the_radius");
}
