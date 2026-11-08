use std::cell::RefCell;
use std::rc::Rc;

use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::{capture, ControlsExt};
use carbide::animation::AnimationManager;
use carbide::animation::keyframes::{Keyframes, Repeat, TimelineController, TimelineHandle};
use carbide::color::PURPLE;
use carbide::draw::{Angle, Dimension};
use carbide::state::{LocalState, ReadStateExtNew, RepeatMode};
use carbide::time::*;
use carbide::widget::*;
use carbide::{Application, Window};

/// Infinite loading spinner, showcasing `Repeat::forever` and the imperative
/// controller's `play`/`stop` — started and later stopped from two different
/// buttons. Handle lifetime is managed with a plain `Rc<RefCell<Option<..>>>`
/// (ordinary Rust, not a carbide `State`) shared between the two closures.
fn main() {
    let mut application = Application::new().with_asset_fonts();

    let angle_degrees = LocalState::new(0.0f64);
    let spinner: Rc<RefCell<Option<TimelineHandle>>> = Rc::new(RefCell::new(None));

    application.set_scene(Window::new(
        "Keyframe repeat - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Rectangle::new()
                .fill(PURPLE)
                .frame(60.0, 60.0)
                .rotation_effect(angle_degrees.map(|d: &f64| Angle::Degrees(*d))),
            HStack::new((
                Button::new("Start spinning", capture!({spinner, angle_degrees}, |ctx: MouseAreaActionContext| {
                    if let Some(handle) = spinner.borrow().as_ref() {
                        handle.play();
                        return;
                    }
                    let spin = Repeat::forever(
                        Keyframes::new(angle_degrees.clone(), 0.0).to(360.0, Duration::from_millis(900)),
                        RepeatMode::FromBeginning,
                    );
                    if let Some(manager) = ctx.env.get_mut::<AnimationManager>() {
                        *spinner.borrow_mut() = Some(manager.insert_controller(TimelineController::new(spin)));
                    }
                }))
                    .frame(140.0, 22.0),
                Button::new("Stop spinning", capture!({spinner}, |_ctx: MouseAreaActionContext| {
                    if let Some(handle) = spinner.borrow().as_ref() {
                        handle.stop();
                    }
                }))
                    .frame(140.0, 22.0),
            )).spacing(10.0),
        )).spacing(20.0)
            .button_style(BorderedProminentStyle),
    ));

    application.launch();
}
