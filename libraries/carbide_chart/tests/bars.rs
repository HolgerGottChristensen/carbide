//! Bar charts: bars growing from zero, grouped datasets splitting a slot, stacked bars, horizontal
//! bars, and floating bars from a low to a high value.

use carbide_chart::{Axis, AxisPosition, BarController, CategoryScale, Chart, DataColor, DatasetExt, LinearScale, Range, RangeDataSetExt, ScaleId, StackId};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::COSTS;

/// A plain bar chart: one dataset of positive values, nothing else on the chart. Pins down the
/// single-dataset bar width, the x scale's offset at both ends, and bars growing up from zero at
/// the bottom of the y scale.
#[test]
fn single_dataset() {
    let chart = Chart::new(BarController::new(vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0])).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("single_dataset");
}

/// One dataset with both positive and negative values. Pins down that every bar starts at zero,
/// in the middle of the y scale: positive bars grow up from it, negative ones hang down from it.
#[test]
fn positive_and_negative() {
    let chart = Chart::new(BarController::new(vec![-12.0, 19.0, -3.0, 5.0, -2.0, 8.0])).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("positive_and_negative");
}

/// One dataset of only negative values. Zero is still included in the y scale, so it is the top
/// of the scale and every bar hangs down from it.
#[test]
fn all_negative() {
    let chart = Chart::new(BarController::new(vec![-5.0, -12.0, -8.0, -20.0, -3.0])).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("all_negative");
}

/// Three grouped datasets with both positive and negative values. Pins down that each group
/// splits its slot three ways in dataset order (blue, green, orange), and that bars on either side
/// of zero in the same group share the zero line.
#[test]
fn grouped_positive_and_negative() {
    let datasets = (
        vec![65.0, -59.0, 80.0, -81.0, 56.0],
        vec![-28.0, 48.0, -40.0, 19.0, 86.0],
        vec![35.0, -10.0, 22.0, 60.0, -45.0],
    );
    let chart = Chart::new(BarController::new(datasets)).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("grouped_positive_and_negative");
}

/// Bars at numeric x values (hours of the day) on an explicit linear x scale. Pins down the
/// linear scale's offset of half a tick step at both ends, and bar widths taken from the smallest
/// gap between bars or ticks.
#[test]
fn on_linear_x_scale() {
    // Visitors per four-hour block of a day, as bars at numeric x values (the hour) on a
    // linear x scale.
    let visitors = vec![(0.0, 3.0), (4.0, 1.0), (8.0, 12.0), (12.0, 18.0), (16.0, 15.0), (20.0, 9.0), (24.0, 4.0)];

    let chart = Chart::new(BarController::new(visitors))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("on_linear_x_scale");
}

/// Materials and labour stacked into one costs bar, beside a revenue bar on its own. Pins down
/// that a stack takes one share of each month's slot, like a single dataset, so there are two
/// bars per month; that the first dataset in the stack is at the bottom (materials, blue, under
/// labour, green); and that the y scale reaches the stacked costs, not just each dataset's values.
#[test]
fn stacked() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![40.0, 45.0, 50.0, 42.0, 55.0].stack(COSTS);
    let labour = vec![60.0, 62.0, 70.0, 65.0, 80.0].stack(COSTS);
    let revenue = vec![120.0, 115.0, 150.0, 98.0, 160.0];

    let chart = Chart::new(BarController::new((materials, labour, revenue))).framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked");
}

/// Three datasets of mixed signs in one stack. Pins down that positive values stack upwards from
/// zero and negative values downwards, each on their own: a negative value in the middle dataset
/// hangs from zero, or from the negatives below it, never from the top of the positive bars.
#[test]
fn stacked_positive_and_negative() {
    const NET: StackId = StackId::new("net");

    let income = vec![20.0, 15.0, 30.0, 10.0, 25.0].stack(NET);
    let expenses = vec![-12.0, -18.0, -10.0, -25.0, 8.0].stack(NET);
    let adjustments = vec![5.0, -4.0, 8.0, -3.0, -6.0].stack(NET);

    let chart = Chart::new(BarController::new((income, expenses, adjustments))).framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_positive_and_negative");
}

/// Two named stacks side by side in one controller: last year's and this year's sales, each
/// stacked from the same two products. Pins down that each stack takes its own share of the slot,
/// so there are two bars per month; that datasets join only the stack they name, whatever their
/// order in the controller; and that each product keeps the palette colour it picks
/// (`DataColor::Palette`) in both stacks.
#[test]
fn two_stacks_side_by_side() {
    const THIS_YEAR: StackId = StackId::new("this year");
    const LAST_YEAR: StackId = StackId::new("last year");
    // Sales of two products, stacked per year, with the datasets interleaved: this year's
    // product A comes between last year's two products. Each product picks its palette colour
    // itself (`DataColor::Palette`), A the first and B the second, so it keeps that colour in
    // both years.
    let last_year_a = vec![30.0, 35.0, 40.0, 38.0, 45.0].stack(LAST_YEAR).color(DataColor::Palette(0));
    let this_year_a = vec![36.0, 40.0, 48.0, 50.0, 55.0].stack(THIS_YEAR).color(DataColor::Palette(0));
    let last_year_b = vec![20.0, 18.0, 25.0, 22.0, 30.0].stack(LAST_YEAR).color(DataColor::Palette(1));
    let this_year_b = vec![25.0, 28.0, 30.0, 35.0, 33.0].stack(THIS_YEAR).color(DataColor::Palette(1));

    let chart = Chart::new(BarController::new((last_year_a, this_year_a, last_year_b, this_year_b)))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("two_stacks_side_by_side");
}

/// Datasets of different lengths in one stack: the bottom dataset has five months, the one on top
/// only three. Pins down that points are matched by x value: the top dataset stacks on the
/// bottom one where both have a value, and in the months it has no value the stack is just the
/// bottom bar.
#[test]
fn stacked_of_different_lengths() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![40.0, 45.0, 50.0, 42.0, 55.0].stack(COSTS);
    let labour = vec![60.0, 62.0, 70.0].stack(COSTS);

    let chart = Chart::new(BarController::new((materials, labour))).framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_of_different_lengths");
}

/// Materials and labour stacked on the default y scale, and units produced with the same stack id
/// on a second y scale on the right. Pins down that a stack is the datasets with the same id on
/// the same scales: the units aren't added onto the costs, but stand as their own bar beside the
/// costs stack, measured on the right-hand scale, which only reaches the units.
#[test]
fn same_stack_on_two_y_scales() {
    const UNITS: ScaleId = ScaleId::new("units");
    // Materials and labour in the costs stack on Y, and units produced, also in the costs stack,
    // but on a `units` scale on the right.
    let materials = vec![40.0, 45.0, 50.0, 42.0, 55.0].stack(COSTS);
    let labour = vec![60.0, 62.0, 70.0, 65.0, 80.0].stack(COSTS);
    let units = vec![1800.0, 2100.0, 2600.0, 1500.0, 3000.0].stack(COSTS).y_scale(UNITS);

    let chart = Chart::new(BarController::new((materials, labour, units)))
        .scale(UNITS, LinearScale::new(Axis::Vertical).position(AxisPosition::Right).grid_lines(false))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("same_stack_on_two_y_scales");
}

/// Two grouped datasets with missing values: revenue has no value in the second month, costs
/// none in the fourth. Pins down that a missing value has no bar but keeps its place in the slot,
/// so the other dataset's bar stays where it is, and that the missing values add nothing to the
/// y scale.
#[test]
fn with_missing_values() {
    let revenue = vec![Some(12.0), None, Some(3.0), Some(5.0), Some(2.0)];
    let costs = vec![Some(8.0), Some(11.0), Some(6.0), None, Some(3.0)];

    let chart = Chart::new(BarController::new((revenue, costs))).framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("with_missing_values");
}

/// Two datasets with positive and negative values as horizontal bars, on the default scales. Pins
/// down that the categories run down a vertical category scale, the first at the top, with grid
/// lines between the groups and half a step of offset at both ends; that each group splits its
/// slot with the first dataset on top; and that the bars grow from zero along the x scale, right
/// for positive values and left for negative ones.
#[test]
fn horizontal() {
    let revenue = vec![12.0, 19.0, -3.0, 5.0, 2.0];
    let costs = vec![8.0, -11.0, 6.0, 4.0, 3.0];

    let chart = Chart::new(BarController::new((revenue, costs)).horizontal(true))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal");
}

/// Two stacked datasets with positive and negative values and one dataset on its own, as
/// horizontal bars. Pins down that a stack lies along the x scale, positive values stacking right
/// of zero and negative ones left, each on their own, the first dataset nearest zero; and that the
/// stack and the dataset on its own split each slot, the stack on top.
#[test]
fn horizontal_stacked() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![10.0, 20.0, 15.0, 8.0].stack(COSTS);
    let labour = vec![5.0, -12.0, 6.0, 9.0].stack(COSTS);
    let revenue = vec![18.0, 14.0, -6.0, 20.0];

    let chart = Chart::new(BarController::new((materials, labour, revenue)).horizontal(true))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_stacked");
}

/// Horizontal bars on a vertical category scale with labels, and a titled value scale. Pins down
/// that a category scale set with `Chart::scale` names the rows from the top, and still gets the
/// bars' offset and grid lines between the rows.
#[test]
fn horizontal_with_labels() {
    let visitors = vec![1200.0, 950.0, 640.0, 410.0, 180.0];

    let chart = Chart::new(BarController::new(visitors).horizontal(true))
        .scale(ScaleId::Y, CategoryScale::new(Axis::Vertical).labels(["Search", "Direct", "Social", "Email", "Referral"]))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).title("Visitors"))
        .framed();

    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("horizontal_with_labels");
}

/// A month's lowest and highest temperature as a floating bar. Pins down that each bar spans from
/// its low to its high value instead of growing from zero, and that the y scale fits the ranges
/// without reaching down to zero.
#[test]
fn floating() {
    let temperatures = vec![
        Range { x: 0.0, low: 4.0, high: 12.0 },
        Range { x: 1.0, low: 6.0, high: 15.0 },
        Range { x: 2.0, low: 9.0, high: 19.0 },
        Range { x: 3.0, low: 11.0, high: 23.0 },
        Range { x: 4.0, low: 10.0, high: 21.0 },
    ];

    let chart = Chart::new(BarController::new(temperatures.range())).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("floating");
}

/// Floating bars, some of whose highs are below their lows, with an inverted colour. Pins down
/// that only the inverted bars take the inverted colour, and that they span the same values as if
/// their ends were the other way round.
#[test]
fn floating_inverted() {
    let changes = vec![
        Range { x: 0.0, low: 4.0, high: 10.0 },
        Range { x: 1.0, low: 10.0, high: 7.0 },
        Range { x: 2.0, low: 7.0, high: 13.0 },
        Range { x: 3.0, low: 13.0, high: 5.0 },
        Range { x: 4.0, low: 5.0, high: 9.0 },
    ];

    let chart = Chart::new(BarController::new(changes.range().inverted_color(DataColor::Palette(3)))).framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("floating_inverted");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::StackId;

    pub const COSTS: StackId = StackId::new("costs");
}
