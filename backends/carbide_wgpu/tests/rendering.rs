use carbide_controls::toggle::{Toggle, ToggleValue};
use carbide_core::draw::theme::Theme;
use carbide_core::draw::{Color, Dimension, Position};
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::widget::{Rectangle, WidgetExt, ZStack};
use carbide_wgpu::testing::{Harness, WgpuMsaa};
use carbide_winit::event::MouseButton;

#[test]
fn rectangle_basic() {
    let widget = Rectangle::new()
        .fill(EnvironmentColor::Blue)
        .frame(20.0, 20.0);

    let mut harness = Harness::new(widget, Dimension::new(40.0, 40.0));

    harness.snapshot("rectangle_basic");
}

#[test]
fn rectangle_mouse_states() {
    let widget = Rectangle::new()
        .fill(Color::new_rgba(0, 0, 255, 255))
        .frame(200.0, 100.0);

    let mut harness = Harness::new(widget, Dimension::new(200.0, 100.0));
    let center = Position::new(100.0, 50.0);

    harness.snapshot("rectangle_idle");

    harness.mouse_move(center);
    harness.mouse_press(MouseButton::Left);
    harness.snapshot("rectangle_pressed");

    harness.mouse_release(MouseButton::Left);
    harness.snapshot("rectangle_released");
}

/// Exercises `Harness::new_with_options` with a non-1.0 scale factor and MSAA enabled — both
/// were previously hardcoded internally (scale factor to 1.0 in four places, MSAA to `X1`), and
/// the physical-pixel-size computation never actually multiplied by scale factor at all, so a
/// `Harness` constructed with `scale_factor: 2.0` would have silently read back a texture sized
/// as if it were 1.0. This checks both the readback dimensions and the actual pixel content are
/// correct at a non-default scale factor, with multisampling enabled.
#[test]
fn rectangle_scale_factor_and_msaa() {
    let widget = Rectangle::new()
        .fill(Color::new_rgba(0, 0, 255, 255))
        .frame(100.0, 50.0);

    let mut harness = Harness::new_with_options(widget, Dimension::new(100.0, 50.0), 2.0, WgpuMsaa::X4);
    harness.render();
    let pixels = harness.pixels();

    assert_eq!(pixels.width(), 200, "physical width should be logical width * scale_factor");
    assert_eq!(pixels.height(), 100, "physical height should be logical height * scale_factor");

    let center = *pixels.get_pixel(100, 50);
    assert_eq!(center.0, [0, 0, 255, 255], "center pixel should be pure blue, unaffected by MSAA");

    harness.snapshot("rectangle_scale_2x_msaa_4x");
}

/// Drives the same `Toggle` through two independent harnesses — one via the recommended mouse
/// pattern (position derived from `find_by_label`, not hardcoded), one via
/// `invoke_accessibility_action` — and snapshots both against the *same* golden file. If the
/// accessibility action path ever stops matching the mouse path's effect, only this test's
/// accessibility-driven half fails, pinpointing exactly that class of regression.
#[test]
fn toggle_accessible_and_mouse_paths_agree() {
    let mouse_value = LocalState::new(ToggleValue::False);
    let widget = Toggle::new("Click me", mouse_value).frame(120.0, 40.0);

    let mut mouse_harness = Harness::new_with_options(widget, Dimension::new(120.0, 40.0), 2.0, WgpuMsaa::X4);
    let (_, node) = mouse_harness.find_by_label("Click me").expect("toggle should be labeled");
    let bounds = node.bounds().expect("toggle node should have bounds");
    let center = Position::new((bounds.x0 + bounds.x1) / 2.0, (bounds.y0 + bounds.y1) / 2.0);

    mouse_harness.mouse_move(center);
    mouse_harness.mouse_press(MouseButton::Left);
    mouse_harness.mouse_release(MouseButton::Left);
    mouse_harness.snapshot("toggle_clicked");

    let a11y_value = LocalState::new(ToggleValue::False);
    let widget = Toggle::new("Click me", a11y_value).frame(120.0, 40.0);

    let mut a11y_harness = Harness::new_with_options(widget, Dimension::new(120.0, 40.0), 2.0, WgpuMsaa::X4);
    // A real screen-reader interaction focuses a control before activating it; mirror that so
    // the comparison isn't confounded by focus-ring styling differences unrelated to the actual
    // action-dispatch parity this test is checking.
    a11y_harness.invoke_accessibility_action("Click me", accesskit::Action::Focus);
    a11y_harness.invoke_accessibility_action("Click me", accesskit::Action::Click);
    a11y_harness.snapshot("toggle_clicked"); // same golden file as the mouse path above

    a11y_harness.snapshot_accessibility_tree("toggle_tree");
    a11y_harness.snapshot_render_instructions("toggle_render_plan");
}

/// `Harness::set_theme` should reach the root `ThemeManager` (added in `new_with_options`, see
/// harness.rs) exactly like `Window::new`'s does, so a widget filled with an `EnvironmentColor`
/// (rather than a literal `Color`) resolves to a different concrete color per theme.
#[test]
fn set_theme_changes_system_background() {
    let widget = Rectangle::new()
        .fill(EnvironmentColor::SystemBackground)
        .frame(100.0, 50.0);

    let mut harness = Harness::new(widget, Dimension::new(100.0, 50.0));

    harness.set_theme(Theme::Dark);
    harness.render();
    let dark = *harness.pixels().get_pixel(50, 25);
    harness.snapshot("theme_dark_background");

    harness.set_theme(Theme::Light);
    harness.render();
    let light = *harness.pixels().get_pixel(50, 25);
    harness.snapshot("theme_light_background");

    assert_ne!(dark, light, "switching theme should change the resolved SystemBackground color");
}

fn shadow_widget() -> impl carbide_core::widget::Widget {
    ZStack::new((
        Rectangle::new().fill(Color::new_rgba(0, 0, 0, 255)), // black background
        Rectangle::new()
            .fill(Color::new_rgba(0, 255, 0, 255)) // green
            .frame(60.0, 60.0)
            .shadow(6.0, Color::new_rgba(255, 0, 255, 255), 8, 8), // sigma, magenta color, offset_x, offset_y
    ))
}

/// Shadow rendering routes through its own offscreen Gaussian-blur filter pass
/// (`Shadow::render`, `ImageFilter::gaussian_blur_1d`) rather than sampling the main
/// multisampled color target directly, so switching MSAA sample count is exactly the kind of
/// change that could desync the shadow pass from the main pipeline. This renders the same
/// shadowed rectangle at 1x MSAA; its 4x sibling below renders the identical scene for visual
/// comparison.
#[test]
fn shadow_msaa_1x() {
    let widget = shadow_widget();
    let mut harness = Harness::new_with_options(widget, Dimension::new(200.0, 200.0), 1.0, WgpuMsaa::X1);
    harness.render();

    let center = *harness.pixels().get_pixel(100, 100);
    assert_eq!(center.0, [0, 255, 0, 255], "rectangle's own fill should be unaffected by its shadow");

    harness.snapshot("shadow_msaa_1x");
}

/// See `shadow_msaa_1x` — identical scene, rendered at 4x MSAA instead.
#[test]
fn shadow_msaa_4x() {
    let widget = shadow_widget();
    let mut harness = Harness::new_with_options(widget, Dimension::new(200.0, 200.0), 1.0, WgpuMsaa::X4);
    harness.render();

    let center = *harness.pixels().get_pixel(100, 100);
    assert_eq!(center.0, [0, 255, 0, 255], "rectangle's own fill should be unaffected by its shadow");

    harness.snapshot("shadow_msaa_4x");
}
