use std::time::Duration;

use carbide_core::color::{BLUE, GREEN, RED, Color};
use carbide_core::animation::{bounce_out, ease_in_out, linear, elastic_in_out};
use carbide_core::state::{LocalState, TState, ReadState, State, TransitionState};
use carbide_core::{a, animate};
use carbide_controls::capture;
use carbide_controls::Button;
use carbide_core::environment::{Environment, EnvironmentFontSize};
use carbide_core::widget::*;
use carbide_core::draw::Dimension;
use carbide_wgpu::{Application, Window};

use carbide_core as carbide;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let switch = LocalState::new(false);

    application.set_scene(Window::new(
        "On change - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Text::new(switch.clone())
                .font_size(EnvironmentFontSize::Title),
            Button::new_primary("Change", a!(|_, _| { *$switch = !*$switch; }))
                .frame(96.0, 22.0),
        )).spacing(10.0)
            .on_change(switch.clone(),a!(|old, new| {
                println!("old: {:?}, new: {:?}", old, new);
            }))
    ).close_application_on_window_close());

    application.launch();
}