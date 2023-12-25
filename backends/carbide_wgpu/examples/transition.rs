use std::time::Duration;

use carbide_core::color::{BLUE, GREEN, RED, Color};
use carbide_core::animation::{bounce_out, ease_in_out, linear, elastic_in_out};
use carbide_core::state::{LocalState, TState, ReadState, State, TransitionState};
use carbide_core::{a, animate};
use carbide_controls::capture;
use carbide_controls::Button;
use carbide_core::environment::Environment;
use carbide_core::widget::*;
use carbide_core::draw::Dimension;
use carbide_wgpu::{Application, Window};

use carbide_core as carbide;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let offset_x = LocalState::new(-120.0);

    let transition = TransitionState::new(offset_x).duration(Duration::new(2, 0));
    let mut transition1 = transition.clone();
    let mut transition2 = transition.clone();

    application.set_scene(Window::new(
        "Transition - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Rectangle::new()
                .frame(60.0, 60.0)
                .offset(transition.clone(), 0.0),
            HStack::new((
                Button::new_primary("Left", a!(|_, _| {
                    transition1.clone().set_value(-120.0)
                }))
                    .frame(96.0, 22.0),
                Button::new_primary("Right", a!(|_, _| {
                    transition2.clone().set_value(120.0)
                }))
                    .frame(96.0, 22.0),
            )).spacing(10.0),
        )).spacing(10.0)
    ).close_application_on_window_close());


    application.launch();
}