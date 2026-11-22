//! Polar charts with too little to draw: no data, too few categories for a loop or a polygon,
//! ranges with no width, gaps, a chart too small for its labels, a sweep with no width, a hole too
//! large, values a log radius can't place, and datasets that can't be drawn around the circle.

use carbide_chart::{Axis, CategoryScale, Chart, DatasetExt, Fill, LineController, LinearScale, LogScale, ScaleId, ScatterController, SpanGaps, StackId};
use carbide_core::draw::{Angle, Dimension, Scalar};
use carbide_testing::Harness;

mod common;

use common::Framed;

const SKILLS: [&str; 6] = ["Speed", "Strength", "Stamina", "Passing", "Shooting", "Defence"];

/// A radar whose datasets are empty, one stacked and filled, on a category angle scale with
/// labels. Pins down that it renders without panicking: the labels still go around the rim with
/// their spokes and polygon rings, the radius spans the default `0..1`, and nothing is drawn
/// inside the circle.
#[test]
fn empty_datasets() {
    const STACK: StackId = StackId::new("stack");
    let empty: Vec<Scalar> = vec![];

    let chart = Chart::new(LineController::new((empty.clone().stack(STACK).fill(Fill::Stack), empty.fill(Fill::Bottom))))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("empty_datasets");
}

/// Radars with one and two categories. Pins down that one point draws no line, only its point,
/// on the single spoke at the top; that two points close into a chord there and back, whose fill
/// has no area; and that with fewer than three corners the rings are circles, though the line
/// controller asks for polygons.
#[test]
fn too_few_categories() {
    let chart = Chart::new(LineController::new(vec![5.0].fill(Fill::Bottom)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["Only"]))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("one_category");

    let chart = Chart::new(LineController::new(vec![3.0, 8.0].fill(Fill::Bottom)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["North", "South"]))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("two_categories");
}

/// A radar whose values are all equal, and a polar scatter of a single point, so their scales
/// start out with no width. Pins down that the radius is widened around the value, as a vertical
/// scale is, putting the loop halfway out as a regular hexagon; and that a single point widens a
/// linear angle scale too, the point at its middle, halfway round from the top.
#[test]
fn ranges_with_no_width() {
    let chart = Chart::new(LineController::new(vec![6.0; 6]))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("equal_values");

    let chart = Chart::new(ScatterController::new(vec![(90.0, 4.0)]))
        .scale(ScaleId::X, LinearScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("single_scatter_point");
}

/// Radars with missing values, filled to the centre. Pins down that a gap in the middle breaks
/// the loop, so the line isn't closed and each run is filled down to the centre on its own; that
/// a missing last value breaks the line where it would close, though the rest is one run; and
/// that `SpanGaps::Always` joins across both, closing the loop again.
#[test]
fn gaps() {
    let middle = vec![Some(8.0), Some(6.0), None, Some(9.0), Some(5.0), Some(7.0)].fill(Fill::Bottom);
    let last = vec![Some(4.0), Some(5.0), Some(3.0), Some(4.0), Some(5.0), None];
    let spanned = vec![Some(2.0), None, Some(3.0), Some(2.0), Some(3.0), None].span_gaps(SpanGaps::Always);

    let chart = Chart::new(LineController::new((middle, last, spanned)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("gaps");
}

/// A radar too small for its long labels. Pins down that the circle shrinks for the labels only
/// down to half the radius it would have without them, and stays centred, the labels then
/// running out of the chart area rather than the circle disappearing.
#[test]
fn too_small_for_its_labels() {
    let labels = ["A very long category name", "Another long one", "Short", "And one more that is long", "Mid", "Last"];

    let chart = Chart::new(LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0]))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(labels))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(240.0, 200.0));

    harness.snapshot("too_small_for_its_labels");
}

/// An angle scale whose end is its start, so it sweeps nothing. Pins down that it renders without
/// panicking: every category falls on the one spoke at the top, the circle is as large as its
/// spoke allows, the line runs up and down that spoke without closing, and the rings are arcs of
/// no length, with only the spoke drawn.
#[test]
fn sweep_with_no_width() {
    let chart = Chart::new(LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0]))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS).end_angle(Angle::Degrees(0.0)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("sweep_with_no_width");
}

/// A hole larger than the circle, and one below zero. Pins down that the inner radius is kept
/// between none and 0.95 of the radius, so a ratio of 2 leaves a thin ring at the rim to draw in,
/// and a ratio of -1 draws no hole.
#[test]
fn hole_out_of_range() {
    let chart = Chart::new(LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].fill(Fill::Bottom)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).inner_radius(2.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("hole_larger_than_the_circle");

    let chart = Chart::new(LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0].fill(Fill::Bottom)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).inner_radius(-1.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("hole_below_zero");
}

/// A radar on a log radius with a zero and a negative value, which have no place on it. Pins down
/// that those two points aren't drawn and break the line, as missing values do, so it doesn't
/// close, and that the radius starts at the smallest positive value.
#[test]
fn log_radius_without_a_place_for_some_values() {
    let chart = Chart::new(LineController::new(vec![20.0, 0.0, 300.0, 5_000.0, -40.0, 60.0]))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .scale(ScaleId::Y, LogScale::new(Axis::Radial))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("log_radius_without_a_place_for_some_values");
}

/// Datasets that can't be drawn around the circle, next to a radar that can: one on a radial x
/// scale and a vertical y scale, and one whose horizontal controller asks for the radar's x scale
/// to be the radius, after the radar asked for it to be the angle. Pins down that both are left
/// out, each logged once, and that the radar, the first to ask, is drawn as it would be alone.
#[test]
fn datasets_that_cannot_be_drawn() {
    let side = ScaleId::new("side");

    let radar = LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0]);
    let mixed = LineController::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0].y_scale(side.clone()));
    let conflicting = LineController::new(vec![5.0, 5.0, 5.0, 5.0, 5.0, 5.0]).horizontal(true);

    let chart = Chart::new((radar, mixed, conflicting))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .scale(side, LinearScale::new(Axis::Vertical))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("datasets_that_cannot_be_drawn");
}

/// A lying line, which asks for its x scale to be the radius, before scatter points on the same
/// scales, which ask for it to be the angle. Pins down that the first to ask wins, so the lying
/// line is drawn with its categories out along the radius, and that the scatter points, whose
/// scales took the other roles, are left out, logged once, rather than drawn in the wrong place.
#[test]
fn scatter_points_whose_scales_took_the_other_roles() {
    let lying = LineController::new(vec![7.3, 6.1, 6.8, 8.6, 5.2, 4.4]).horizontal(true);
    let points = ScatterController::new(vec![(1.0, 5.0), (3.0, 7.0), (4.0, 6.0)]);

    let chart = Chart::new((lying, points))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(SKILLS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("scatter_points_whose_scales_took_the_other_roles");
}
