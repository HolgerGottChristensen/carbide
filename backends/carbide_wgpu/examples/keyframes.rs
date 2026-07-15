use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_controls::{capture, ControlsExt};
use carbide_core::animation::AnimationManager;
use carbide_core::animation::{bounce_out};
use carbide_core::animation::keyframes::{Keyframes, Repeat, TimelineController, Tracks};
use carbide_core::color::{Color, BLUE, GREEN, RED};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, RepeatMode};
use carbide_core::time::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

/// Direct multi-stop analogue of `animate_on_click.rs`, using the new
/// composable keyframe/timeline engine instead of the single-tween
/// `animate!` macro: one click plays a 3-stop offset track and a 2-stop
/// color track in parallel (a `Tracks` of two `Keyframes`).
fn main() {
    let mut application = Application::new().with_asset_fonts();

    let offset_x = LocalState::new(-120.0);
    let offset_y = LocalState::new(-120.0);
    let color = LocalState::new(RED);

    application.set_scene(Window::new(
        "Keyframes - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Spacer::new(),
            Rectangle::new()
                .fill(color.clone())
                .frame(60.0, 60.0)
                .offset(offset_x.clone(), offset_y.clone()),
            Button::new("Bounce", capture!({offset_x, color}, |ctx: MouseAreaActionContext| {
                let motion = Repeat::counted(Tracks::new()
                    .and(Keyframes::new(offset_x.clone(), -120.0)
                        .to(120.0, Duration::from_millis(600))
                        .to_with_curve(-0.0, Duration::from_millis(400), bounce_out)
                        .to(-120.0, Duration::from_millis(300)))
                    .and(Keyframes::new(offset_y.clone(), -120.0)
                        .to_with_curve(-60.0, Duration::from_millis(400), bounce_out)
                        .to(0.0, Duration::from_millis(600))
                        .to(-120.0, Duration::from_millis(300)))
                    .and(Keyframes::new(color.clone(), RED)
                        .to_with_interpolation(BLUE, Duration::from_secs(1), Color::hsla_blend)
                        .to(GREEN, Duration::from_millis(300))), RepeatMode::Alternate, 2);

                if let Some(manager) = ctx.env.get_mut::<AnimationManager>() {
                    manager.insert_controller(TimelineController::new(motion));
                }
            }))
                .frame(96.0, 22.0),
        )).spacing(20.0)
            .button_style(BorderedProminentStyle)
            .padding(20.0),
    ));

    application.launch();
}
