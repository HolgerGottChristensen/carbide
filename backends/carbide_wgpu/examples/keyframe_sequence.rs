use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_controls::{capture, ControlsExt};
use carbide_core::animation::AnimationManager;
use carbide_core::animation::{ease_in, ease_out};
use carbide_core::animation::keyframes::{Chain, Delay, Keyframes, TimelineController};
use carbide_core::color::GREEN;
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::time::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

/// Multi-stop analogue of `transition.rs`: a toast slides in, holds, then
/// slides back out, using `Chain::then()` with a `Delay` in the middle for
/// the "hold" step — the gap `Keyframes<T>` alone can't express.
fn main() {
    let mut application = Application::new().with_asset_fonts();

    let toast_offset = LocalState::new(100.0);

    application.set_scene(Window::new(
        "Keyframe sequence - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Button::new("Show toast", capture!({toast_offset}, |ctx: MouseAreaActionContext| {
                let toast = Chain::new()
                    .then(Keyframes::new(toast_offset.clone(), 100.0)
                        .to_with_curve(0.0, Duration::from_millis(300), ease_out))
                    .then(Delay::new(Duration::from_millis(1200)))
                    .then(Keyframes::new(toast_offset.clone(), 0.0)
                        .to_with_curve(100.0, Duration::from_millis(300), ease_in));

                if let Some(manager) = ctx.env.get_mut::<AnimationManager>() {
                    manager.insert_controller(TimelineController::new(toast));
                }
            }))
                .frame(120.0, 22.0),
            Rectangle::new()
                .fill(GREEN)
                .frame(200.0, 50.0)
                .offset(0.0, toast_offset.clone()),
        )).spacing(20.0)
            .button_style(BorderedProminentStyle),
    ));

    application.launch();
}
