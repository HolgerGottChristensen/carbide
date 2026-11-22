//! Bars around a circle: a bar controller on two radial scales. Upright, the categories go around
//! the circle and the values out along the radius, each bar a wedge; lying down (`horizontal`),
//! the categories run out along the radius and the values around the circle, radial bars. Covers
//! the slots, stacking and grouping, the base at zero, the ring shapes, and the marks that draw
//! around a circle.

use carbide_chart::{Axis, BarController, BoxDataSetExt, BoxStats, CategoryScale, Chart, DataColor, DatasetExt, Estimate, GridShape, LegendPosition, LineController, LinearScale, Range, RangeDataSetExt, ScaleId, StackId};
use carbide_core::draw::{Angle, Dimension};
use carbide_testing::Harness;

mod common;

use common::Framed;

const MONTHS: [&str; 8] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"];

/// A polar area chart: one dataset of bars around a category angle scale, coloured by point.
/// Pins down that each bar is a wedge from the centre, at zero, out to its value's ring; that the
/// half-category offset bars ask for centres each wedge in its slot, the first a half slot past
/// the top; that the controller's percentages leave gaps between wedges; that the spokes fall
/// between the categories, from the grid offset bars ask for; and that the rings are circles.
#[test]
fn polar_area() {
    let rainfall = vec![42.0, 35.0, 48.0, 60.0, 55.0, 70.0, 66.0, 50.0].color(DataColor::ByPoint);

    let chart = Chart::new(BarController::new(rainfall))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(MONTHS))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("polar_area");
}

/// Wedges filling their whole slots, with a hole in the middle: a nightingale chart. Pins down
/// that `category_percentage` and `bar_percentage` of 1 make the wedges touch, and that the bars'
/// base, zero, is the inner radius, so the wedges start at the hole.
#[test]
fn touching_wedges_around_a_hole() {
    let visits = vec![42.0, 35.0, 48.0, 60.0, 55.0, 70.0, 66.0, 50.0].color(DataColor::ByPoint);

    let chart = Chart::new(BarController::new(visits).category_percentage(1.0).bar_percentage(1.0))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(MONTHS))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).inner_radius(0.3))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("touching_wedges_around_a_hole");
}

/// Grouped and stacked wedges: a dataset of its own next to a stack of two, with a legend. Pins
/// down that each category's slot is split between the group's members around the circle, as
/// along a side; that stacked bars are rings of the same slice, the second from the first's end;
/// and that the legend shows a square per dataset.
#[test]
fn grouped_and_stacked_wedges() {
    const COSTS: StackId = StackId::new("costs");

    let revenue = vec![12.0, 19.0, 14.0, 17.0, 22.0, 18.0].label("Revenue");
    let wages = vec![6.0, 7.0, 7.0, 8.0, 9.0, 8.0].stack(COSTS).label("Wages");
    let rent = vec![3.0, 3.0, 3.0, 4.0, 4.0, 4.0].stack(COSTS).label("Rent");

    let chart = Chart::new(BarController::new((revenue, wages, rent)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["Jan", "Feb", "Mar", "Apr", "May", "Jun"]))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("grouped_and_stacked_wedges");
}

/// Radial bars: `horizontal`, so the categories run out along the radius and the values around
/// the circle, on an angle scale fixed from 0 to 100, with a hole. Pins down that each bar is an
/// arc of a ring, its share of its category's slot out along the radius, from zero at the top
/// round to its value; that 100 is a full turn; that a value past 100 stops at a full turn rather
/// than going round again; and that the categories' labels sit beside the start spoke.
#[test]
fn radial_bars() {
    let progress = vec![35.0, 60.0, 82.0, 100.0, 130.0].color(DataColor::ByPoint);

    let chart = Chart::new(BarController::new(progress).horizontal())
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["Design", "Build", "Test", "Docs", "Release"]))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).min(0.0).max(100.0).step(25.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("radial_bars");
}

/// Wedges for values either side of zero, on a half circle across the top. Pins down that bars
/// start at the ring at zero, inside the radius, which runs from below zero: positive bars grow
/// out from it, negative bars in towards the centre; and that a partial sweep fits its arc.
#[test]
fn values_either_side_of_zero_on_a_half_circle() {
    let change = vec![4.0, -2.0, 6.0, -3.5, 2.5, 5.0];

    let chart = Chart::new(BarController::new(change))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["A", "B", "C", "D", "E", "F"])
            .start_angle(Angle::Degrees(-90.0))
            .end_angle(Angle::Degrees(90.0)))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 400.0));

    harness.snapshot("values_either_side_of_zero_on_a_half_circle");
}

/// Bars beside a radar on the same scales. Pins down that both draw around one circle, the
/// radar's points at the centres of the bars' slots, offset half a category as the bars ask; and
/// that the bars' circles win over the radar's polygons for the rings.
#[test]
fn bars_beside_a_radar() {
    let bars = BarController::new(vec![3.0, 5.0, 4.0, 6.0, 2.0, 5.0]);
    let radar = LineController::new(vec![8.0, 6.0, 7.0, 9.0, 5.0, 4.0]).palette_offset(1);

    let chart = Chart::new((bars, radar))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["Speed", "Strength", "Stamina", "Passing", "Shooting", "Defence"]))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("bars_beside_a_radar");
}

/// Wedges around polygon rings, set on the radius scale. Pins down that a wedge's outer and inner
/// edges follow the grid's ring shape, straight from corner to corner of the polygon through
/// the categories, as steps and fills do, instead of arcs.
#[test]
fn wedges_around_polygons() {
    let visits = vec![42.0, 35.0, 48.0, 60.0, 55.0, 70.0].color(DataColor::ByPoint);

    let chart = Chart::new(BarController::new(visits).category_percentage(1.0).bar_percentage(1.0))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["N", "NE", "SE", "S", "SW", "NW"]))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).grid_shape(GridShape::Polygon))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("wedges_around_polygons");
}

/// Floating bars and error bars around a circle, upright and lying down. Pins down that a
/// floating bar is a wedge from its low's ring to its high's, or, lying down, an arc from its
/// low's angle to its high's; and that error bars run along the middle spoke of their bar's
/// share, caps across it as wide as that share of the wedge where they are, or, lying down,
/// around the ring through the bar's middle, caps across the ring.
#[test]
fn floating_bars_and_error_bars() {
    let ranges = vec![
        Range { x: 0.0, low: 2.0, high: 6.0 },
        Range { x: 1.0, low: 4.0, high: 9.0 },
        Range { x: 2.0, low: 1.0, high: 5.0 },
        Range { x: 3.0, low: 3.0, high: 8.0 },
        Range { x: 4.0, low: 5.0, high: 7.0 },
    ].range();
    let estimates = vec![
        Estimate { x: 0.0, y: 5.0, low: 4.0, high: 6.5 },
        Estimate { x: 1.0, y: 7.0, low: 6.0, high: 8.0 },
        Estimate { x: 2.0, y: 4.0, low: 3.0, high: 5.5 },
        Estimate { x: 3.0, y: 6.0, low: 5.0, high: 7.5 },
        Estimate { x: 4.0, y: 6.5, low: 5.5, high: 7.0 },
    ].error_bars();
    let labels = ["A", "B", "C", "D", "E"];

    let chart = Chart::new(BarController::new((ranges.clone(), estimates.clone())))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(labels))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).min(0.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("upright");

    let chart = Chart::new(BarController::new((ranges, estimates)).horizontal())
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(labels))
        .scale(ScaleId::Y, LinearScale::new(Axis::Radial).min(0.0).max(10.0))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.snapshot("lying_down");
}

/// Box plots next to bars around a circle. Pins down that box plots, which don't draw around a
/// circle yet, are left out, logged once, while the bars beside them are drawn, in their share of
/// the slot as if the boxes were there.
#[test]
fn box_plots_left_out() {
    let boxes = vec![
        BoxStats { x: 0.0, min: 2.0, q1: 3.0, median: 4.0, q3: 5.0, max: 7.0 },
        BoxStats { x: 1.0, min: 1.0, q1: 2.5, median: 3.5, q3: 4.0, max: 6.0 },
        BoxStats { x: 2.0, min: 3.0, q1: 4.0, median: 5.0, q3: 6.0, max: 8.0 },
        BoxStats { x: 3.0, min: 2.0, q1: 3.5, median: 4.5, q3: 5.5, max: 6.5 },
    ].box_plot();
    let bars = vec![4.0, 3.0, 6.0, 5.0];

    let chart = Chart::new(BarController::new((boxes, bars)))
        .scale(ScaleId::X, CategoryScale::new(Axis::Radial).labels(["A", "B", "C", "D"]))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("box_plots_left_out");
}
