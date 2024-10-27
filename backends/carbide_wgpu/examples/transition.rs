use std::time::Duration;

use carbide_controls::Button;
use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, ReadStateExtTransition, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let offset = LocalState::new(-120.0);

    let transition = offset.transition().duration(Duration::new(2, 0));

    application.set_scene(Window::new(
        "Transition - Carbide",
        Dimension::new(400.0, 300.0),
        VStack::new((
            Rectangle::new()
                .frame(60.0, 60.0)
                .offset(transition, 0.0),
            HStack::new((
                Button::new_primary("Left", closure!(|_| {
                    *$offset = -120.0;
                }))
                    .frame(96.0, 22.0),
                Button::new_primary("Right", closure!(|_| {
                    *$offset = 120.0;
                }))
                    .frame(96.0, 22.0),
            )).spacing(10.0),
        )).spacing(10.0)
    ).close_application_on_window_close());


    application.launch();
}