//! The pointer in a chart, driven through the harness as a user would: hovering reports and
//! outlines the data under the pointer, clicks call the click action, and on zoomable scales a
//! drag and the wheel pan, a pinch and Ctrl with the wheel zoom, a Shift-drag zooms to a box, a
//! double-click resets, and smart zoom toggles; a zoomable scale's band under the pointer is
//! tinted, and a Shift-drag along it selects a span of that scale.

use std::cell::RefCell;
use std::rc::Rc;
use carbide_chart::{Axis, BarController, CategoryScale, Chart, DatumKey, LinearScale, LineController, Measure, Pick, PickMode, ScaleId, ScatterController, ZoomWindow};
use carbide_core::draw::{Dimension, Position};
use carbide_core::event::ModifierKey;
use carbide_core::state::{LocalState, ReadState};
use carbide_core::widget::{Scroll, VStack, WidgetExt};
use carbide_testing::Harness;

mod common;

use common::Framed;
use support::{key, wave_line, zoom_states};

const QUARTERS: [&str; 4] = ["Q1", "Q2", "Q3", "Q4"];

/// Grouped bars with a hover state, the pointer over the empty space above Q3's bars, then moved
/// off the chart area. Pins down that hovering finds both datasets' bars at Q3, the nearest
/// category (the default `Index` mode), outlines them, and reports the values under the pointer,
/// Q3 on x; and that leaving the chart area reports nothing.
#[test]
fn hovering_finds_and_outlines_the_data_at_the_nearest_category() {
    let hovered = LocalState::new(None::<Pick>);
    let chart = Chart::new(BarController::new((vec![10.0, 20.0, 30.0, 25.0], vec![15.0, 25.0, 5.0, 20.0])))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(QUARTERS))
        .hover(hovered.clone())
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(380.0, 120.0));
    harness.snapshot("hovering_finds_and_outlines_the_data_at_the_nearest_category");

    let pick = hovered.value().clone().expect("the pointer is over the chart area");
    assert_eq!(pick.data, vec![key(0, 0, 2), key(0, 1, 2)]);
    let category = pick.value(&ScaleId::X).and_then(|value| value.category.clone()).expect("x has categories");
    assert_eq!(category.name.as_deref(), Some("Q3"));

    harness.mouse_move(Position::new(5.0, 5.0));
    assert_eq!(*hovered.value(), None, "off the chart area");
}

/// Scattered points with hovering by the nearest point. Pins down that `hover_mode` changes how
/// hovering finds data: the one point nearest the pointer is outlined.
#[test]
fn hover_mode_picks_how_data_is_found() {
    let hovered = LocalState::new(None::<Pick>);
    let chart = Chart::new(ScatterController::new((vec![(1.0, 2.0), (3.0, 8.0), (5.0, 3.0)], vec![(2.0, 9.0), (4.0, 1.0)])))
        .hover(hovered.clone())
        .hover_mode(PickMode::Nearest(Measure::Both))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(340.0, 170.0));
    harness.snapshot("hover_mode_picks_how_data_is_found");

    assert_eq!(hovered.value().as_ref().map(|pick| pick.data.clone()), Some(vec![key(0, 0, 1)]));
}

/// Bars with a click action. Pins down that a click in the chart area calls it with what was
/// found there, and a click outside it doesn't.
#[test]
fn clicks_report_what_is_under_the_pointer() {
    let clicked: Rc<RefCell<Vec<Vec<DatumKey>>>> = Rc::new(RefCell::new(vec![]));
    let record = clicked.clone();
    let chart = Chart::new(BarController::new((vec![10.0, 20.0, 30.0, 25.0], vec![15.0, 25.0, 5.0, 20.0])))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(QUARTERS))
        .hover_mode(PickMode::Point)
        .on_click(move |click| record.borrow_mut().push(click.pick.data.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    // On the first dataset's Q2 bar, then on the empty space above Q1, then outside the area.
    harness.click(Position::new(225.0, 400.0));
    harness.click(Position::new(120.0, 80.0));
    harness.click(Position::new(5.0, 5.0));

    assert_eq!(*clicked.borrow(), vec![vec![key(0, 0, 1)], vec![]]);
}

/// A line zoomable along x, zoomed into its second quarter, the wheel turned sideways, then down.
/// Pins down that the wheel pans as a drag does, the data moving the way the wheel moves a scroll
/// view's content: sideways, later values come into view; and that turning it down, which would
/// move y, which shows its whole range and can't move, changes nothing.
#[test]
fn the_wheel_pans_like_a_drag() {
    let x = LocalState::new(ZoomWindow::new(0.25, 0.5));
    let (_, y) = zoom_states();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(y.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(300.0, 250.0));
    harness.scroll(-100.0, 0.0);
    harness.snapshot("the_wheel_pans_like_a_drag");

    let window = *x.value();
    assert!(window.start > 0.25 && (window.span() - 0.25).abs() < 1e-9, "moved on, keeping its width: {window:?}");

    harness.scroll(0.0, 60.0);
    assert_eq!(*x.value(), window, "a vertical wheel doesn't move x");
    assert_eq!(*y.value(), ZoomWindow::WHOLE, "nor y, which shows its whole range");
}

/// A line zoomable along both scales, the wheel turned three lines towards zooming in with Ctrl
/// held and the pointer over the chart area, and a second chart with the same over its x scale's
/// band. Pins down that with Ctrl the wheel zooms instead of panning, so a mouse can zoom: over
/// the chart area both scales about the pointer, by a tenth a line, keeping the value under it in
/// place; and over a band only that scale.
#[test]
fn ctrl_and_the_wheel_zoom_about_the_pointer() {
    let (x, y) = zoom_states();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(y.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.set_modifiers(ModifierKey::CONTROL);
    harness.mouse_move(Position::new(200.0, 200.0));
    harness.scroll(0.0, 30.0);
    harness.snapshot("over_the_chart_area");

    let expected = 1.1_f64.powi(-3);
    assert!((x.value().span() - expected).abs() < 1e-9, "x zoomed in by three lines: {:?}", *x.value());
    assert!((y.value().span() - expected).abs() < 1e-9, "y too: {:?}", *y.value());

    let (x, y) = zoom_states();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(y.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.set_modifiers(ModifierKey::CONTROL);
    harness.mouse_move(Position::new(300.0, 465.0));
    harness.scroll(0.0, 30.0);
    // Onto the chart area before the snapshot, so x's band isn't tinted as hovered.
    harness.mouse_move(Position::new(300.0, 250.0));
    harness.snapshot("over_the_x_band");

    assert!(x.value().span() < 1.0, "x zoomed");
    assert_eq!(*y.value(), ZoomWindow::WHOLE, "y left as it is");
}

/// A line zoomable along both scales, with the pointer over x's band below the chart area, then
/// over the chart area. Pins down that the hovered scale's band is tinted, its tick marks, labels
/// and border drawn as always; and that moving onto the chart area takes the tint away.
#[test]
fn hovering_a_scale_tints_its_band() {
    let (x, y) = zoom_states();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(y))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(300.0, 465.0));
    harness.snapshot("over_the_x_band");

    harness.mouse_move(Position::new(300.0, 250.0));
    harness.snapshot("over_the_chart_area");
}

/// A line zoomable along both scales, Shift-dragged along x's band from the left to the middle.
/// Pins down that the selection is drawn as a span along x, across its band only, rather than a
/// box over the chart area, and that its band stays tinted while dragging; and that letting go, still
/// over the band, zooms only x to it, y keeping its whole range, the band tinted again.
#[test]
fn shift_dragging_along_a_scale_zooms_it_to_a_span() {
    let (x, y) = zoom_states();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(y.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.set_modifiers(ModifierKey::SHIFT);
    harness.mouse_move(Position::new(150.0, 465.0));
    harness.mouse_press(carbide_testing::MouseButton::Left);
    harness.mouse_move(Position::new(220.0, 465.0));
    harness.mouse_move(Position::new(320.0, 460.0));
    harness.snapshot("drawing_the_span");

    harness.mouse_release(carbide_testing::MouseButton::Left);
    harness.set_modifiers(ModifierKey::empty());
    harness.snapshot("zoomed_to_the_span");

    let window = *x.value();
    assert!(window.start > 0.1 && window.end < 0.6, "x zoomed to the span: {window:?}");
    assert_eq!(*y.value(), ZoomWindow::WHOLE, "y left as it is");
}

/// A line zoomed into its second quarter along x, dragged to the right. Pins down that dragging
/// pans the zoomed scale, the data following the pointer, so earlier values come into view; and
/// that the y scale, showing its whole range, doesn't move.
#[test]
fn dragging_pans() {
    let (x, y) = zoom_states();
    let mut x = x;
    *carbide_core::state::State::value_mut(&mut x) = ZoomWindow::new(0.25, 0.5);
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(y.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.drag(Position::new(300.0, 250.0), Position::new(400.0, 250.0), 4);
    harness.snapshot("dragging_pans");

    let window = *x.value();
    assert!(window.start < 0.25 && (window.span() - 0.25).abs() < 1e-9, "moved back, keeping its width: {window:?}");
    assert_eq!(*y.value(), ZoomWindow::WHOLE);
}

/// A line zoomable along both scales, Shift-dragged from the upper left to the middle. Pins down
/// that the box is drawn while dragging, and that letting go zooms both scales to it.
#[test]
fn shift_dragging_zooms_to_a_box() {
    let (x, y) = zoom_states();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .scale(ScaleId::Y, LinearScale::new(Axis::Vertical).zoom(y.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.set_modifiers(ModifierKey::SHIFT);
    harness.mouse_move(Position::new(120.0, 80.0));
    harness.mouse_press(carbide_testing::MouseButton::Left);
    harness.mouse_move(Position::new(200.0, 160.0));
    harness.mouse_move(Position::new(320.0, 260.0));
    harness.snapshot("drawing_the_box");

    harness.mouse_release(carbide_testing::MouseButton::Left);
    harness.set_modifiers(ModifierKey::empty());
    harness.snapshot("zoomed_to_the_box");

    assert!(x.value().span() < 0.5 && y.value().span() < 0.5, "{:?} {:?}", *x.value(), *y.value());
}

/// A line zoomable along x, pinched to zoom in, then double-clicked. Pins down that a pinch zooms
/// about the pointer, and a double-click shows the whole range again.
#[test]
fn a_pinch_zooms_and_a_double_click_resets() {
    let (x, _) = zoom_states();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(400.0, 250.0));
    harness.pinch(1.0);
    harness.snapshot("pinched");
    assert!((x.value().span() - 0.5).abs() < 1e-9, "a pinch of 1 halves the span: {:?}", *x.value());

    harness.double_click(Position::new(400.0, 250.0));
    harness.snapshot("double_clicked");
    assert_eq!(*x.value(), ZoomWindow::WHOLE);
}

/// A chart zoomed along x in a scroll view, above a chart without a zoom, the wheel turned
/// sideways, then down, over the zoomed chart. Pins down that the chart takes the wheel when it
/// pans, so the view doesn't move; and that turning it down, which the chart can't pan along, as
/// y isn't zoomable, scrolls the view as usual.
#[test]
fn a_chart_takes_the_wheel_from_a_scroll_view_only_when_it_pans() {
    let x = LocalState::new(ZoomWindow::new(0.25, 0.5));
    let zoomed = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .frame(600.0, 300.0);
    let plain = Chart::new(BarController::new(vec![10.0, 20.0, 30.0, 25.0])).frame(600.0, 300.0);

    let view = Scroll::new(VStack::new((zoomed, plain))).framed();
    let mut harness = Harness::new(view, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(300.0, 150.0));
    harness.scroll(-100.0, 0.0);
    harness.snapshot("sideways_over_the_zoomed_chart");
    assert!(x.value().start > 0.25, "the chart panned: {:?}", *x.value());

    // Down, so the view, at its top, has somewhere to scroll.
    let panned = *x.value();
    harness.scroll(0.0, -60.0);
    harness.snapshot("down_over_the_zoomed_chart");
    assert_eq!(*x.value(), panned, "the chart can't pan down");
}

/// A line zoomable along x, given the trackpad's smart zoom twice. Pins down that showing the
/// whole range, smart zoom zooms in by half about the pointer, and zoomed in, it shows the whole
/// range again.
#[test]
fn smart_zoom_toggles_between_zoomed_in_and_the_whole_range() {
    let (x, _) = zoom_states();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(400.0, 250.0));
    harness.smart_scale();
    harness.snapshot("zoomed_in");
    assert!((x.value().span() - 0.5).abs() < 1e-9, "half the range: {:?}", *x.value());

    harness.smart_scale();
    assert_eq!(*x.value(), ZoomWindow::WHOLE);
}

/// A zoomable bar chart with a hover state and a click action, its interactivity a state turned
/// off while hovered, then on again. Pins down that turning it off lets go of the hovered data and
/// its outline, and that while off the wheel, a drag and a click do nothing; and that turned on
/// again, the chart responds as before.
#[test]
fn interactivity_can_be_turned_off() {
    let mut interactive = LocalState::new(true);
    let hovered = LocalState::new(None::<Pick>);
    let zoom = LocalState::new(ZoomWindow::new(0.25, 0.75));
    let clicks = Rc::new(RefCell::new(0));
    let count = clicks.clone();

    let chart = Chart::new(BarController::new((vec![10.0, 20.0, 30.0, 25.0], vec![15.0, 25.0, 5.0, 20.0])))
        .scale(ScaleId::X, CategoryScale::new(Axis::Horizontal).labels(QUARTERS).zoom(zoom.clone()))
        .hover(hovered.clone())
        .on_click(move |_| *count.borrow_mut() += 1)
        .interactive(interactive.clone())
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    harness.mouse_move(Position::new(300.0, 120.0));
    assert!(hovered.value().is_some(), "hovered while interactive");

    *carbide_core::state::State::value_mut(&mut interactive) = false;
    harness.snapshot("turned_off_while_hovered");
    assert_eq!(*hovered.value(), None, "turning it off lets go of the hovered data");

    harness.mouse_move(Position::new(320.0, 130.0));
    harness.scroll(-100.0, 0.0);
    harness.drag(Position::new(300.0, 250.0), Position::new(450.0, 250.0), 3);
    harness.click(Position::new(300.0, 250.0));
    assert_eq!(*hovered.value(), None, "not hovered");
    assert_eq!(*zoom.value(), ZoomWindow::new(0.25, 0.75), "not panned");
    assert_eq!(*clicks.borrow(), 0, "not clicked");

    *carbide_core::state::State::value_mut(&mut interactive) = true;
    harness.mouse_move(Position::new(300.0, 120.0));
    harness.click(Position::new(300.0, 120.0));
    assert!(hovered.value().is_some(), "hovered again");
    assert_eq!(*clicks.borrow(), 1, "clicked again");
}

/// Two charts zoomed along x, one above the other, dragged from the lower chart up over the upper
/// one. Pins down that only a press inside a chart drags it: the lower chart pans, following the
/// pointer wherever it goes, while the upper chart, which the drag passes over but didn't start
/// in, keeps its window.
#[test]
fn only_a_press_inside_a_chart_drags_it() {
    let (upper_zoom, lower_zoom) = (LocalState::new(ZoomWindow::new(0.25, 0.5)), LocalState::new(ZoomWindow::new(0.25, 0.5)));
    let upper = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(upper_zoom.clone()))
        .frame(600.0, 250.0);
    let lower = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(lower_zoom.clone()))
        .frame(600.0, 250.0);

    let mut harness = Harness::new(VStack::new((upper, lower)).framed(), Dimension::new(600.0, 500.0));
    harness.render();

    harness.drag(Position::new(300.0, 375.0), Position::new(400.0, 125.0), 6);
    harness.snapshot("only_a_press_inside_a_chart_drags_it");

    assert_eq!(*upper_zoom.value(), ZoomWindow::new(0.25, 0.5), "the drag didn't start in the upper chart");
    assert!(lower_zoom.value().start < 0.25, "the lower chart panned: {:?}", *lower_zoom.value());
}

/// A zoomed chart with a click action, dragged away and back to where the press was, then
/// clicked. Pins down that a press that moved isn't a click, even when it is let go where it
/// started; and that the click after it is one, though carbide counts it as the second click of
/// a double-click, which doesn't reset the zoom, as its first press was a drag.
#[test]
fn a_drag_back_to_its_start_isnt_a_click() {
    let x = LocalState::new(ZoomWindow::new(0.25, 0.5));
    let clicks = Rc::new(RefCell::new(0));
    let count = clicks.clone();
    let chart = Chart::new(LineController::new(wave_line()))
        .scale(ScaleId::X, LinearScale::new(Axis::Horizontal).zoom(x.clone()))
        .on_click(move |_| *count.borrow_mut() += 1)
        .framed();
    let mut harness = Harness::new(chart, Dimension::new(600.0, 500.0));
    harness.render();

    let start = Position::new(300.0, 250.0);
    harness.mouse_move(start);
    harness.mouse_press(carbide_testing::MouseButton::Left);
    harness.mouse_move(Position::new(400.0, 250.0));
    harness.mouse_move(start);
    harness.mouse_release(carbide_testing::MouseButton::Left);
    assert_eq!(*clicks.borrow(), 0, "the press moved, so it isn't a click");

    harness.click(start);
    assert_eq!(*clicks.borrow(), 1);
    assert_ne!(*x.value(), ZoomWindow::WHOLE, "not a double-click, so the zoom stays");
}

// ===============================
//  Utility
// ===============================

mod support {
    use carbide_chart::{DatumKey, ZoomWindow};
    use carbide_core::state::LocalState;

    pub fn key(controller: usize, dataset: usize, index: usize) -> DatumKey {
        DatumKey { controller, dataset, index }
    }

    /// Zoom states for x and y, each showing the whole range.
    pub fn zoom_states() -> (LocalState<ZoomWindow>, LocalState<ZoomWindow>) {
        (LocalState::new(ZoomWindow::WHOLE), LocalState::new(ZoomWindow::WHOLE))
    }

    /// Sixty points of a smooth wave with a slower swell.
    pub fn wave_line() -> Vec<(f64, f64)> {
        (0..60).map(|i| {
            let x = i as f64;
            (x, 50.0 + 30.0 * (x / 5.0).sin() + 15.0 * (x / 17.0).cos())
        }).collect()
    }
}
