use carbide_controls::Button;
use carbide_core::a;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, ReadStateExtNew, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

use carbide_core as carbide;

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

            Button::new_primary("Add 1", a!(|_,_| { *$counter_state += 1; }))
                .frame(90.0, 22.0),

            Button::new("Subtract 1", a!(|_,_| { *$counter_state -= 1; }))
                .frame(90.0, 22.0),

            Button::new("Disabled", a!(|_,_|{}))
                .enabled(false)
                .frame(90.0, 22.0),
        ))
            .spacing(20.0)
    ).close_application_on_window_close());

    application.launch();
}
