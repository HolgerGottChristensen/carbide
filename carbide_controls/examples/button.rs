use carbide_controls::{Button, capture};
use carbide_core::draw::Dimension;
use carbide_core::environment::Environment;
use carbide_core::focus::Focus;
use carbide_core::state::{LocalState, ReadStateExtNew, State, StateExt};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter_state = LocalState::new(0);

    application.set_scene(Window::new(
        "Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        *VStack::new(vec![
            Text::new(counter_state.map(|count: &i32| format!("Count: {}", count)))
                .font_size(40u32),
            Button::new("Add 1", capture!([counter_state], |env: &mut Environment| {
                *counter_state.value_mut() += 1;
            }))
                .frame(90.0, 22.0),
            Button::new("Subtract 1", capture!([counter_state], |env: &mut Environment| {
                *counter_state.value_mut() -= 1;
            }))
                .frame(90.0, 22.0),
        ])
            .spacing(20.0)
    ).close_application_on_window_close());

    application.launch();
}
