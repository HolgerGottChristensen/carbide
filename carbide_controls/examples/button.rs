use carbide_controls::{Button, ControlsExt};
use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, ReadStateExtNew, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

use carbide_core as carbide; // Required only in internal examples

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter_state = LocalState::new(0);

    application.set_scene(Window::new(
        "Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Text::new(counter_state.map(|count: &i32| format!("Count: {}", count)))
                .font_size(32u32),

            Button::new_primary("Add 1", closure!(|_,_| { *$counter_state += 1; }))
                .frame(90.0, 22.0),

            Button::new("Subtract 1", closure!(|_,_| { *$counter_state -= 1; }))
                .frame(90.0, 22.0),

            /*Button::new(Image::new("images/landscape.png").scaled_to_fill(), a!(|_,_| {}))
                .frame(90.0, 22.0),*/

            Button::new("Disabled", closure!(|_,_|{}))
                .enabled(false)
                .frame(90.0, 22.0),

            HStack::new((
                Button::new_primary(Image::new_icon("icons/chat-1-line.png"), closure!(|_,_| {}))
                    .frame(32.0, 32.0),
                Button::new(Image::new_icon("icons/chat-1-line.png"), closure!(|_,_| {}))
                    .frame(32.0, 32.0),
                Button::new(Image::new_icon("icons/chat-1-line.png"), closure!(|_,_| {}))
                    .enabled(false)
                    .frame(32.0, 32.0),
            )).spacing(10.0)
        ))
            .spacing(20.0)
    ).close_application_on_window_close());

    application.launch();
}
