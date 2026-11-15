//! Bar charts on their own: bars growing from zero, grouped datasets splitting a slot, stacked
//! bars, and bar widths on a linear x scale.

use carbide_chart::{DatasetExt, StackId};
use carbide_core::draw::Dimension;
use carbide_testing::Harness;

mod common;

use common::bar_chart;
use support::{bars_on_linear_x_scale_chart, two_stacks_chart};

/// A plain bar chart: one dataset of positive values, nothing else on the chart. Pins down the
/// single-dataset bar width, the x scale's offset at both ends, and bars growing up from zero at
/// the bottom of the y scale.
#[test]
fn bar_chart_single_dataset() {
    let mut harness = Harness::new(bar_chart(vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_single_dataset");
}

/// One dataset with both positive and negative values. Pins down that every bar starts at zero,
/// in the middle of the y scale: positive bars grow up from it, negative ones hang down from it.
#[test]
fn bar_chart_positive_and_negative() {
    let mut harness = Harness::new(bar_chart(vec![-12.0, 19.0, -3.0, 5.0, -2.0, 8.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_positive_and_negative");
}

/// One dataset of only negative values. Zero is still included in the y scale, so it is the top
/// of the scale and every bar hangs down from it.
#[test]
fn bar_chart_all_negative() {
    let mut harness = Harness::new(bar_chart(vec![-5.0, -12.0, -8.0, -20.0, -3.0]), Dimension::new(600.0, 500.0));

    harness.snapshot("bar_chart_all_negative");
}

/// Three grouped datasets with both positive and negative values. Pins down that each group
/// splits its slot three ways in dataset order (white, green, red), and that bars on either side
/// of zero in the same group share the zero line.
#[test]
fn grouped_bars_positive_and_negative() {
    let datasets = (
        vec![65.0, -59.0, 80.0, -81.0, 56.0],
        vec![-28.0, 48.0, -40.0, 19.0, 86.0],
        vec![35.0, -10.0, 22.0, 60.0, -45.0],
    );
    let mut harness = Harness::new(bar_chart(datasets), Dimension::new(600.0, 500.0));

    harness.snapshot("grouped_bars_positive_and_negative");
}

/// Bars at numeric x values (hours of the day) on an explicit linear x scale. Pins down the
/// linear scale's offset of half a tick step at both ends, and bar widths taken from the smallest
/// gap between bars or ticks.
#[test]
fn bars_on_linear_x_scale() {
    let mut harness = Harness::new(bars_on_linear_x_scale_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("bars_on_linear_x_scale");
}

/// Materials and labour stacked into one costs bar, beside a revenue bar on its own. Pins down
/// that a stack takes one share of each month's slot, like a single dataset, so there are two
/// bars per month; that the first dataset in the stack is at the bottom (materials, white, under
/// labour, green); and that the y scale reaches the stacked costs, not just each dataset's values.
#[test]
fn stacked_bars() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![40.0, 45.0, 50.0, 42.0, 55.0].stack(COSTS);
    let labour = vec![60.0, 62.0, 70.0, 65.0, 80.0].stack(COSTS);
    let revenue = vec![120.0, 115.0, 150.0, 98.0, 160.0];

    let mut harness = Harness::new(bar_chart((materials, labour, revenue)), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_bars");
}

/// Three datasets of mixed signs in one stack. Pins down that positive values stack upwards from
/// zero and negative values downwards, each on their own: a negative value in the middle dataset
/// hangs from zero, or from the negatives below it, never from the top of the positive bars.
#[test]
fn stacked_bars_positive_and_negative() {
    const NET: StackId = StackId::new("net");

    let income = vec![20.0, 15.0, 30.0, 10.0, 25.0].stack(NET);
    let expenses = vec![-12.0, -18.0, -10.0, -25.0, 8.0].stack(NET);
    let adjustments = vec![5.0, -4.0, 8.0, -3.0, -6.0].stack(NET);

    let mut harness = Harness::new(bar_chart((income, expenses, adjustments)), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_bars_positive_and_negative");
}

/// Two named stacks side by side in one controller: last year's and this year's sales, each
/// stacked from the same two products. Pins down that each stack takes its own share of the slot,
/// so there are two bars per month; that datasets join only the stack they name, whatever their
/// order in the controller; and that each stack's products keep their own colours.
#[test]
fn two_stacks_side_by_side() {
    let mut harness = Harness::new(two_stacks_chart(), Dimension::new(600.0, 500.0));

    harness.snapshot("two_stacks_side_by_side");
}

/// Datasets of different lengths in one stack: the bottom dataset has five months, the one on top
/// only three. Pins down that points are matched by x value: the top dataset stacks on the
/// bottom one where both have a value, and in the months it has no value the stack is just the
/// bottom bar.
#[test]
fn stacked_bars_of_different_lengths() {
    const COSTS: StackId = StackId::new("costs");

    let materials = vec![40.0, 45.0, 50.0, 42.0, 55.0].stack(COSTS);
    let labour = vec![60.0, 62.0, 70.0].stack(COSTS);

    let mut harness = Harness::new(bar_chart((materials, labour)), Dimension::new(600.0, 500.0));

    harness.snapshot("stacked_bars_of_different_lengths");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{Axis, BarController, Chart, DataColor, DatasetExt, LinearScale, ScaleId, StackId};
    use carbide_core::color::{GREEN, WHITE};
    use carbide_core::widget::Widget;
    use crate::common::Framed;

    const LAST_YEAR: StackId = StackId::new("last year");
    const THIS_YEAR: StackId = StackId::new("this year");

    /// Sales of two products, stacked per year, with the datasets interleaved: this year's
    /// product A comes between last year's two products. Product A is white and B green in both
    /// years.
    pub fn two_stacks_chart() -> impl Widget {
        let last_year_a = vec![30.0, 35.0, 40.0, 38.0, 45.0].stack(LAST_YEAR).color(DataColor::Color(WHITE));
        let this_year_a = vec![36.0, 40.0, 48.0, 50.0, 55.0].stack(THIS_YEAR).color(DataColor::Color(WHITE));
        let last_year_b = vec![20.0, 18.0, 25.0, 22.0, 30.0].stack(LAST_YEAR).color(DataColor::Color(GREEN));
        let this_year_b = vec![25.0, 28.0, 30.0, 35.0, 33.0].stack(THIS_YEAR).color(DataColor::Color(GREEN));

        Chart::new(BarController::new((last_year_a, this_year_a, last_year_b, this_year_b)))
            .framed()
    }

    /// Visitors per four-hour block of a day, as bars at numeric x values (the hour) on a
    /// linear x scale.
    pub fn bars_on_linear_x_scale_chart() -> impl Widget {
        let visitors = vec![(0.0, 3.0), (4.0, 1.0), (8.0, 12.0), (12.0, 18.0), (16.0, 15.0), (20.0, 9.0), (24.0, 4.0)];

        Chart::new(BarController::new(visitors))
            .scale(ScaleId::X, LinearScale::new(Axis::Horizontal))
            .framed()
    }
}
