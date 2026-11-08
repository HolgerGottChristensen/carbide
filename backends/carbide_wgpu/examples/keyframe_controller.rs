use std::cell::RefCell;
use std::rc::Rc;

use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_controls::slider::Slider;
use carbide_controls::{capture, ControlsExt};
use carbide_core::animation::AnimationManager;
use carbide_core::animation::keyframes::{Keyframes, TimelineController, TimelineHandle, Tracks};
use carbide_core::color::{Color, BLUE, RED};
use carbide_core::draw::Dimension;
use carbide_core::environment::Environment;
use carbide_core::state::LocalState;
use carbide_core::time::*;
use carbide_core::widget::*;
use carbide_winit::{Application, Window};

/// Imperative playback controls (play/pause/stop/seek) driven through a
/// `TimelineHandle`, with a real `carbide_controls::Slider` wired to `seek`
/// via `.on_change`. The controller is built lazily on first interaction
/// (mirroring `keyframe_repeat.rs`'s pattern) since there is no `Environment`
/// available yet at plain widget-tree construction time in `main()`.
fn main() {
    let mut application = Application::new().with_asset_fonts();

    let offset_x = LocalState::new(-120.0);
    let color = LocalState::new(RED);
    let progress = LocalState::new(0.0f64);
    let handle_cell: Rc<RefCell<Option<TimelineHandle>>> = Rc::new(RefCell::new(None));

    fn ensure_handle(
        cell: &Rc<RefCell<Option<TimelineHandle>>>,
        offset: &LocalState<f64>,
        color: &LocalState<Color>,
        env: &mut Environment,
    ) -> Option<TimelineHandle> {
        if let Some(handle) = cell.borrow().as_ref() {
            return Some(handle.clone());
        }

        let motion = Tracks::new()
            .and(Keyframes::new(offset.clone(), -120.0).to(120.0, Duration::from_secs(2)))
            .and(Keyframes::new(color.clone(), RED)
                .to_with_interpolation(BLUE, Duration::from_secs(2), Color::hsla_blend));

        let controller = TimelineController::new(motion).paused();
        let handle = env.get_mut::<AnimationManager>()?.insert_controller(controller);
        *cell.borrow_mut() = Some(handle.clone());
        Some(handle)
    }

    application.set_scene(Window::new(
        "Keyframe controller - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Rectangle::new()
                .fill(color.clone())
                .frame(60.0, 60.0)
                .offset(offset_x.clone(), 0.0),
            HStack::new((
                Button::new("Play", capture!({handle_cell, offset_x, color}, |ctx: MouseAreaActionContext| {
                    if let Some(handle) = ensure_handle(&handle_cell, &offset_x, &color, ctx.env) {
                        handle.play();
                    }
                }))
                    .frame(90.0, 22.0),
                Button::new("Pause", capture!({handle_cell, offset_x, color}, |ctx: MouseAreaActionContext| {
                    if let Some(handle) = ensure_handle(&handle_cell, &offset_x, &color, ctx.env) {
                        handle.pause();
                    }
                }))
                    .frame(90.0, 22.0),
                Button::new("Stop", capture!({handle_cell, offset_x, color}, |ctx: MouseAreaActionContext| {
                    if let Some(handle) = ensure_handle(&handle_cell, &offset_x, &color, ctx.env) {
                        handle.stop();
                    }
                }))
                    .frame(90.0, 22.0),
            )).spacing(10.0),
            Slider::new(progress.clone(), 0.0, 1.0)
                .padding(20.0)
                .on_change(progress.clone(), capture!({handle_cell}, |_prev: Option<&f64>, new: &f64| {
                    if let Some(handle) = handle_cell.borrow().as_ref() {
                        handle.seek(*new);
                    }
                })),
        )).spacing(20.0)
            .button_style(BorderedProminentStyle),
    ));

    application.launch();
}
