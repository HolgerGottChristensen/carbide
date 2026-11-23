//! Showing and hiding datasets, and a pie's parts: their visibility states (`DatasetExt::visible`,
//! `PieController::hidden_parts`), and clicks on the legend that flip them (`Chart::legend_toggles`).
//! A hidden dataset is left out of the scales' fit, its stack and what is drawn, and its legend
//! entry is dimmed.

use carbide_chart::{Chart, BarController, DatasetExt, Fill, LegendPosition, LineController, PieController, StackId};
use carbide_core::draw::{Dimension, Position};
use carbide_core::state::{LocalState, ReadState};
use carbide_testing::Harness;

mod common;

use common::Framed;

const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];

/// Three grouped datasets with a legend, the tall middle one hidden. Pins down that its bars are
/// gone, the two shown widening to share each quarter's slot, in their own colours; that the y
/// scale refits to the bars shown; and that its legend entry is dimmed.
#[test]
fn a_hidden_bar_dataset() {
    let chart = Chart::new(BarController::new((
        vec![10.0, 14.0, 9.0, 12.0].label("North"),
        vec![40.0, 45.0, 38.0, 50.0].label("South").visible(false),
        vec![8.0, 11.0, 13.0, 10.0].label("West"),
    )))
        .scale(carbide_chart::ScaleId::X, carbide_chart::CategoryScale::new(carbide_chart::Axis::Horizontal).labels(QUARTERS))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("a_hidden_bar_dataset");
}

/// A stacked area of three filled lines, the middle one hidden. Pins down that the line above it
/// moves down onto the first, filling down to it, as it adds nothing to the stack.
#[test]
fn a_hidden_layer_of_a_stacked_area() {
    const TRAFFIC: StackId = StackId::new("traffic");
    let chart = Chart::new(LineController::new((
        vec![30.0, 35.0, 40.0, 30.0, 25.0, 20.0].stack(TRAFFIC).fill(Fill::Bottom).label("Mobile"),
        vec![50.0, 45.0, 40.0, 45.0, 50.0, 50.0].stack(TRAFFIC).fill(Fill::Stack).label("Desktop").visible(false),
        vec![20.0, 20.0, 20.0, 25.0, 25.0, 30.0].stack(TRAFFIC).fill(Fill::Stack).label("Tablet"),
    )))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("a_hidden_layer_of_a_stacked_area");
}

/// A pie with its largest part hidden. Pins down that the parts shown fill the circle, each its
/// share of their total, keeping their colours; that the hidden part has no slice or label; and
/// that its legend entry is dimmed.
#[test]
fn a_hidden_pie_part() {
    let votes = vec![("Red", 12.0), ("Blue", 19.0), ("Yellow", 3.0), ("Green", 5.0)];
    let chart = Chart::new(PieController::new(votes).hidden_parts(vec!["Blue".to_string()]))
        .legend(LegendPosition::Bottom)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));

    harness.snapshot("a_hidden_pie_part");
}

/// The grouped bars of `a_hidden_bar_dataset`, each dataset's visibility a state, with the legend
/// at the bottom: North's entry about (228, 462), South's (300, 462) and West's (380, 462).
fn clickable_bars(shown: &[LocalState<bool>; 3]) -> impl carbide_core::widget::Widget {
    Chart::new(BarController::new((
        vec![10.0, 14.0, 9.0, 12.0].label("North").visible(shown[0].clone()),
        vec![40.0, 45.0, 38.0, 50.0].label("South").visible(shown[1].clone()),
        vec![8.0, 11.0, 13.0, 10.0].label("West").visible(shown[2].clone()),
    )))
        .scale(carbide_chart::ScaleId::X, carbide_chart::CategoryScale::new(carbide_chart::Axis::Horizontal).labels(QUARTERS))
        .legend(LegendPosition::Bottom)
        .framed()
}

fn visible(shown: &[LocalState<bool>; 3]) -> Vec<bool> {
    shown.iter().map(|state| *state.value()).collect()
}

/// A click on South's legend entry, then, after a click elsewhere, a click on it again. Pins down that the click hides
/// South, writing its state, its bars gone and its entry dimmed, as when hidden from the start;
/// and that clicking the dimmed entry shows it again.
#[test]
fn clicking_an_entry_hides_and_shows_its_dataset() {
    let shown = [LocalState::new(true), LocalState::new(true), LocalState::new(true)];
    let mut harness = Harness::new(clickable_bars(&shown), Dimension::new(600.0, 500.0));
    harness.render();

    harness.click(Position::new(300.0, 462.0));
    harness.snapshot("south_hidden");
    assert_eq!(visible(&shown), vec![true, false, true]);

    // A click elsewhere first, so the next click on the entry isn't the second of a double-click.
    harness.click(Position::new(5.0, 5.0));
    harness.click(Position::new(300.0, 462.0));
    assert_eq!(visible(&shown), vec![true, true, true], "shown again");
}

/// With only West shown, a click on its entry. Pins down that the last entry shown can't be
/// hidden.
#[test]
fn the_last_entry_shown_stays() {
    let shown = [LocalState::new(false), LocalState::new(false), LocalState::new(true)];
    let mut harness = Harness::new(clickable_bars(&shown), Dimension::new(600.0, 500.0));
    harness.render();

    harness.click(Position::new(380.0, 462.0));
    assert_eq!(visible(&shown), vec![false, false, true]);
}

/// A double-click on West's entry, then, after a click elsewhere, another. Pins down that the first shows only West, and
/// the second, on the only entry shown, shows them all again.
#[test]
fn double_clicking_shows_only_that_entry() {
    let shown = [LocalState::new(true), LocalState::new(true), LocalState::new(true)];
    let mut harness = Harness::new(clickable_bars(&shown), Dimension::new(600.0, 500.0));
    harness.render();

    harness.double_click(Position::new(380.0, 462.0));
    harness.snapshot("only_west");
    assert_eq!(visible(&shown), vec![false, false, true]);

    // A click elsewhere first, so the next two clicks make a double-click of their own.
    harness.click(Position::new(5.0, 5.0));
    harness.double_click(Position::new(380.0, 462.0));
    assert_eq!(visible(&shown), vec![true, true, true], "every entry again");
}

/// The bars with `legend_toggles(false)`, South's entry clicked. Pins down that the legend then
/// ignores clicks.
#[test]
fn legend_toggles_can_be_turned_off() {
    let shown = [LocalState::new(true), LocalState::new(true), LocalState::new(true)];
    let chart = Chart::new(BarController::new((
        vec![10.0, 14.0].label("North").visible(shown[0].clone()),
        vec![40.0, 45.0].label("South").visible(shown[1].clone()),
        vec![8.0, 11.0].label("West").visible(shown[2].clone()),
    )))
        .legend(LegendPosition::Bottom)
        .legend_toggles(false)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.click(Position::new(300.0, 462.0));
    assert_eq!(visible(&shown), vec![true, true, true]);
}
