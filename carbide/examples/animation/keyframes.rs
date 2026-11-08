use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::{capture, ControlsExt};
use carbide::animation::{ease_in_out, AnimationManager};
use carbide::animation::keyframes::{Keyframes, Repeat, TimelineController, Tracks};
use carbide::color::{Color, BLUE, GREEN, RED};
use carbide::draw::Dimension;
use carbide::state::{LocalState, RepeatMode};
use carbide::time::*;
use carbide::widget::*;
use carbide::{Application, Window};

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
            Button::new("Animate", capture!({offset_x, color}, |ctx: MouseAreaActionContext| {
                let motion = Repeat::counted(Tracks::new()
                    .and(Keyframes::new(offset_x.clone(), -120.0)
                        .to(120.0, Duration::from_millis(600))
                        .to_with_curve(-0.0, Duration::from_millis(1000), ease_in_out)
                        .to(-120.0, Duration::from_millis(300)))
                    .and(Keyframes::new(offset_y.clone(), -120.0)
                        .to_with_curve(-60.0, Duration::from_millis(1000), ease_in_out)
                        .to(0.0, Duration::from_millis(600))
                        .to(-120.0, Duration::from_millis(300)))
                    .and(Keyframes::new(color.clone(), RED)
                        .to_with_interpolation(BLUE, Duration::from_secs(1), Color::hsla_blend)
                        .to(GREEN, Duration::from_millis(900))), RepeatMode::Alternate, 2);

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
