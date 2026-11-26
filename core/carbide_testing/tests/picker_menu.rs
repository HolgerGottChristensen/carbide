//! The menu picker's popup: where its list opens, relative to the window.

use carbide_controls::picker::{MenuStyle, Picker};
use carbide_controls::toggle::Toggle;
use carbide_controls::ControlsExt;
use carbide_core::draw::{Dimension, Position};
use carbide_core::state::{LocalState, ReadState};
use carbide_core::widget::{AnyWidget, HStack, Spacer, Text, VStack, WidgetExt};
use carbide_testing::Harness;
use carbide_winit::event::MouseButton;

/// A menu picker of `count` choices, "Choice 1" and on, at the bottom of a 300 × 300 window, with
/// its popup opened by pressing its button.
fn opened_near_the_bottom(count: usize) -> Harness {
    let choice = LocalState::new(count / 2);
    let picker = Picker::new(
        "Choice",
        choice,
        (1..=count).map(|i| Text::new(format!("Choice {i}")).tag(i - 1)).collect::<Vec<_>>(),
    ).picker_style(MenuStyle);

    let widget = VStack::new((Spacer::new(), picker)).padding(10.0);
    let mut harness = Harness::new(widget, Dimension::new(300.0, 300.0));
    harness.render();

    // The button, on the right of the label, 11 points above the bottom padding. The menu opens on
    // the press; the button is held, as when choosing by dragging to an item and releasing.
    harness.mouse_move(Position::new(200.0, 279.0));
    harness.mouse_press(MouseButton::Left);
    harness.render();
    harness
}

/// A list of 8 choices, centred on a button at the bottom of the window, would run off the bottom.
/// It fits, so it opens moved up, 2 points above the window's bottom edge.
#[test]
fn a_list_that_fits_stays_in_the_window() {
    let mut harness = opened_near_the_bottom(8);
    harness.snapshot("moved_up");
}

/// A list of 30 choices is taller than the window, so it stays centred on its button, running off
/// both edges, until the list can scroll.
#[test]
fn a_list_taller_than_the_window_stays_centred() {
    let mut harness = opened_near_the_bottom(30);
    harness.snapshot("centred");
}

/// A quick click on the button opens the list without choosing: moved up, the list has another
/// choice under the pointer than the chosen one, and the release of the click that opened it
/// doesn't choose it.
#[test]
fn a_click_opens_the_list_without_choosing() {
    let choice = LocalState::new(4usize);
    let picker = Picker::new(
        "Choice",
        choice.clone(),
        (1..=8).map(|i| Text::new(format!("Choice {i}")).tag(i - 1)).collect::<Vec<_>>(),
    ).picker_style(MenuStyle);

    let mut harness = Harness::new(VStack::new((Spacer::new(), picker)).padding(10.0), Dimension::new(300.0, 300.0));
    harness.render();
    // A frame between press and release, as a window draws one: the popup is laid out in it. With
    // none, the release finds the new popup not yet placed, takes it for a click outside the list,
    // and closes it.
    harness.mouse_move(Position::new(200.0, 279.0));
    harness.mouse_press(MouseButton::Left);
    harness.render();
    harness.mouse_release(MouseButton::Left);
    harness.render();

    assert_eq!(*choice.value(), 4, "the click chose another choice");
    harness.snapshot("open_after_click");
}

/// Where a picker sits in its 300 × 300 window.
enum Place {
    Top,
    /// Halfway down, against the left edge.
    Left,
    /// Halfway down, against the right edge.
    Right,
}

/// A menu picker with an empty label, 120 by 22 points, at `place`, of 8 choices named
/// `name(i)`, with its popup opened by pressing its button.
fn opened_at(place: Place, name: impl Fn(usize) -> String) -> Harness {
    let picker = Picker::new(
        "",
        LocalState::new(0usize),
        (1..=8).map(|i| Text::new(name(i)).tag(i - 1)).collect::<Vec<_>>(),
    ).picker_style(MenuStyle).frame(120.0, 22.0);

    // The button follows the empty label and the 8-point spacing, so it spans 112 points.
    let (widget, button): (Box<dyn AnyWidget>, Position) = match place {
        Place::Top => (VStack::new((picker, Spacer::new())).padding(10.0).boxed(), Position::new(154.0, 21.0)),
        Place::Left => (HStack::new((picker, Spacer::new())).boxed(), Position::new(64.0, 150.0)),
        Place::Right => (HStack::new((Spacer::new(), picker)).boxed(), Position::new(244.0, 150.0)),
    };

    let mut harness = Harness::new(widget, Dimension::new(300.0, 300.0));
    harness.render();
    harness.mouse_move(button);
    harness.mouse_press(MouseButton::Left);
    harness.render();
    harness
}

/// A list of 8 choices, centred on a button at the top of the window, would run off the top. It
/// fits, so it opens moved down, 2 points below the window's top edge.
#[test]
fn a_list_at_the_top_moves_down() {
    let mut harness = opened_at(Place::Top, |i| format!("Choice {i}"));
    harness.snapshot("moved_down");
}

/// Choices longer than the button make the list wider than it, on one line each. Centred on a button against the
/// window's left edge, the list would run off it; it fits, so it opens moved right, 2 points
/// from the edge.
#[test]
fn a_wide_list_at_the_left_moves_right() {
    let mut harness = opened_at(Place::Left, |i| format!("A much longer choice {i}"));
    harness.snapshot("moved_right");
}

/// The same against the right edge: the wide list opens moved left, 2 points from the edge.
#[test]
fn a_wide_list_at_the_right_moves_left() {
    let mut harness = opened_at(Place::Right, |i| format!("A much longer choice {i}"));
    harness.snapshot("moved_left");
}

/// Choosing a choice with a click on it, while a control lies under the list where the click is,
/// chooses the choice only: the control under it isn't clicked too. The click's release chooses
/// and closes the list, and the click that follows the release belongs to the list, not to what is
/// under it.
#[test]
fn choosing_a_choice_doesnt_click_what_is_under_it() {
    let choice = LocalState::new(0usize);
    let toggled = LocalState::new(false);
    let picker = Picker::new(
        "",
        choice.clone(),
        (1..=8).map(|i| Text::new(format!("Choice {i}")).tag(i - 1)).collect::<Vec<_>>(),
    ).picker_style(MenuStyle).frame(120.0, 22.0);
    let toggle = Toggle::new("Under the list", toggled.clone()).frame(120.0, 40.0);

    let widget = VStack::new((picker, toggle, Spacer::new())).padding(10.0);
    let mut harness = Harness::new(widget, Dimension::new(300.0, 300.0));
    harness.render();

    // Open the list with a click on the button, a frame drawn between press and release.
    harness.mouse_move(Position::new(158.0, 21.0));
    harness.mouse_press(MouseButton::Left);
    harness.render();
    harness.mouse_release(MouseButton::Left);
    harness.render();
    harness.snapshot("open_over_the_toggle");

    // Click the choice over the toggle.
    let over_the_toggle = Position::new(158.0, 62.0);
    harness.mouse_move(over_the_toggle);
    harness.render();
    harness.mouse_press(MouseButton::Left);
    harness.render();
    harness.mouse_release(MouseButton::Left);
    harness.render();

    assert_ne!(*choice.value(), 0, "the click chose nothing");
    assert!(!*toggled.value(), "the click chose a choice and toggled the toggle under it too");
    harness.snapshot("chosen");
}
