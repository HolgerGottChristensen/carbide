use carbide_controls::toggle::{Toggle, ToggleValue};
use carbide_core::draw::theme::Theme;
use carbide_core::draw::{Color, Dimension, Position, Scalar};
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, State};
use carbide_core::widget::{Rectangle, WidgetExt, ZStack};
use carbide_wgpu::testing::{Harness, WgpuMsaa};
use carbide_winit::event::MouseButton;

#[test]
fn rectangle_basic() {
    let widget = Rectangle::new()
        .fill(EnvironmentColor::Red)
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

/// A shadow's blur radius (`sigma`) and offset are specified in *logical* pixels, like every
/// other widget measurement -- so the shadow's visual size and reach relative to the shape it
/// belongs to should look identical regardless of the window's `scale_factor`, exactly like the
/// shape itself does. This renders the same `shadow_widget()` scene at `scale_factor` 1.0 and
/// 2.0, then samples the shadow's halo at a handful of *logical* positions (mapped to the correct
/// physical pixel for each render via `* scale_factor`) and checks the two agree.
#[test]
fn shadow_blur_radius_is_scale_factor_agnostic() {
    fn render_at(scale_factor: Scalar, snapshot_name: &str) -> image::RgbaImage {
        let widget = shadow_widget();
        let mut harness = Harness::new_with_options(widget, Dimension::new(200.0, 200.0), scale_factor, WgpuMsaa::X1);
        harness.render();
        harness.snapshot(snapshot_name);
        harness.pixels()
    }

    let pixels_1x = render_at(1.0, "shadow_scale_factor_1x");
    let pixels_2x = render_at(2.0, "shadow_scale_factor_2x");

    // Logical points in the shadow's halo below-right of the 60x60 rect (offset (8,8)), chosen to
    // sit on the blur's falloff rather than its fully-saturated interior or its untouched tail, so
    // a scale-factor-dependent radius shows up as a clear intensity difference rather than both
    // sides reading as either full or zero.
    for (logical_x, logical_y) in [(135.0, 135.0), (140.0, 130.0), (130.0, 140.0), (145.0, 145.0), (138.0, 138.0)] {
        let p1 = *pixels_1x.get_pixel(logical_x as u32, logical_y as u32);
        let p2 = *pixels_2x.get_pixel((logical_x * 2.0) as u32, (logical_y * 2.0) as u32);

        let diff = (p1.0[0] as i32 - p2.0[0] as i32).abs();
        assert!(
            diff < 12,
            "shadow intensity at logical ({logical_x},{logical_y}) should match regardless of scale_factor, but 1x={:?} vs 2x={:?} (diff={diff})",
            p1.0, p2.0
        );
    }
}

fn shadow_crossing_border_widget() -> impl carbide_core::widget::Widget {
    ZStack::new((
        Rectangle::new().fill(Color::new_rgba(0, 0, 0, 255)), // black background
        Rectangle::new()
            .fill(Color::new_rgba(0, 255, 0, 255)) // green
            .frame(100.0, 100.0)
            .offset(90.0, 0.0) // push most of the rect off the right edge of the 200-wide window
            .shadow(6.0, Color::new_rgba(255, 0, 255, 255), -80, 0), // sigma, magenta, shifted left, back into the window
    ))
}

/// Only the rectangle's left 60px (x:140-200) are inside the 200x200 window; the remaining 40px
/// (x:200-240) fall off-screen to the right. Its shadow is offset -80 in x, shifting the shadow
/// silhouette left by 80 relative to the rectangle -- far enough that the shadow contribution from
/// the *off-screen* part of the rectangle should land inside the window, in a region (x:120-140)
/// not covered by the rectangle's own visible fill (x:140-200).
///
/// If shadow rendering only samples the window-sized offscreen texture the shape was rasterized
/// into (rather than the shape's true, potentially off-screen, geometry), the off-screen 40px of
/// rectangle never contributes any pixels to the blur, so the shadow silhouette is effectively
/// clipped to the rectangle's *visible* x:140-200 span before the offset is applied -- meaning
/// x:120-140 would incorrectly show no shadow at all.
#[test]
fn shadow_crossing_window_border() {
    let widget = shadow_crossing_border_widget();
    let mut harness = Harness::new(widget, Dimension::new(200.0, 200.0));

    // A shadow's very first `render()` on a freshly constructed `Harness` renders with no
    // shadow at all (confirmed independently: the existing, non-border-crossing `shadow_widget()`
    // scene shows pure background in its shadow region after exactly one `render()`, then the
    // correct blurred shadow from the second `render()` onward) -- a separate, pre-existing
    // warm-up quirk unrelated to the window-border clipping this test targets. This explicit
    // `render()` absorbs that first, unrepresentative frame; `snapshot()` (which renders once
    // more internally) and the `pixels()` read right after it then both reflect the same,
    // stable frame.
    harness.render();
    harness.snapshot("shadow_crossing_window_border");
    let pixels = harness.pixels();

    let own_fill = *pixels.get_pixel(170, 100);
    let far_background = *pixels.get_pixel(20, 100);
    // x:120-140 is uniquely covered by the *true* (unclipped) shadow silhouette but not the
    // *clipped-at-window-edge* one -- the pixel that distinguishes correct from buggy rendering.
    let differential = *pixels.get_pixel(130, 100);

    assert_eq!(own_fill.0, [0, 255, 0, 255], "rectangle's own visible fill should be unaffected by its shadow");
    assert_eq!(far_background.0, [0, 0, 0, 255], "point far from the shadow should remain pure background");
    assert_ne!(
        differential.0, [0, 0, 0, 255],
        "shadow should extend from the off-screen part of the rectangle into this region, but it rendered as pure background -- shadow appears clipped to only the visible part of the shape"
    );
}

fn animated_shadow_widget(offset_x: LocalState<i32>) -> impl carbide_core::widget::Widget {
    ZStack::new((
        Rectangle::new().fill(Color::new_rgba(0, 0, 0, 255)),
        Rectangle::new()
            .fill(Color::new_rgba(0, 255, 0, 255))
            .frame(60.0, 60.0)
            .shadow(6.0, Color::new_rgba(255, 0, 255, 255), offset_x, 8),
    ))
}

/// A shadow's scratch render target is sized to the largest radius (blur spread + offset) any
/// frame has ever asked for on that target index (`TargetStates::get` only ever grows the
/// recorded requirement, since shrinking would need to discard and reallocate a GPU texture every
/// time an animated value dipped). An `AnimatedState`-driven shadow like the `shadow` example
/// (sigma/offset continuously oscillating) reuses that same target every frame, so most frames
/// ask for less padding than the target actually has.
///
/// This renders one harness through a large-offset frame (growing the target), then a small
/// offset (`8`, matching `shadow_widget()`'s convention) matching a second, fresh harness that
/// never saw a large frame. If the transform used to render into the scratch target isn't rebuilt
/// from the target's *actual* size after such a shrink, the two should visibly disagree -- the
/// shrunk-back-down harness's shadow would appear shifted/squished relative to the fresh one,
/// since it'd still assume the small size while actually rendering into the larger, oversized
/// texture left over from the large-offset frame.
#[test]
fn shadow_target_reused_after_radius_shrink_matches_fresh_render() {
    let offset_x = LocalState::new(80);
    let mut harness = Harness::new(animated_shadow_widget(offset_x.clone()), Dimension::new(200.0, 200.0));

    // Warm-up + large-offset frame: absorbs the first-render warm-up quirk (see
    // shadow_crossing_window_border) and grows the scratch target to accommodate offset=80.
    harness.render();

    // Shrink back down to the same small offset `shadow_widget()`/`shadow_msaa_1x` use, reusing
    // the now-oversized target.
    offset_x.clone().set_value(8);
    harness.render();
    harness.snapshot("shadow_target_reused_after_radius_shrink");
    let pixels = harness.pixels();

    // Reference: a harness that only ever renders with the small offset, so its scratch target
    // is never grown beyond what offset=8 actually needs.
    let mut reference_harness = Harness::new(animated_shadow_widget(LocalState::new(8)), Dimension::new(200.0, 200.0));
    reference_harness.render();
    reference_harness.render();
    let reference_pixels = reference_harness.pixels();

    for (x, y) in [(100, 100), (135, 135), (140, 130), (130, 140), (138, 138), (145, 145)] {
        let actual = *pixels.get_pixel(x, y);
        let expected = *reference_pixels.get_pixel(x, y);
        assert_eq!(
            actual.0, expected.0,
            "pixel ({x},{y}) after the shadow's offset shrinks back down should match a harness that never saw a larger offset (target reuse must not leave stale sizing/transform state behind)"
        );
    }
}

