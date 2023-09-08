use carbide_controls::{capture, PlainButton};
use carbide_core::draw::Dimension;
use carbide_core::environment::{Environment};
use carbide_core::state::{LocalState, State, ReadStateExtNew, ReadState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter_state = LocalState::new(0);

    let counter_text = counter_state
        .map(|count: &i32| format!("Count: {}", count));

    let button = PlainButton::new(capture!([counter_state], |env: &mut Environment| {
        let mut temp = counter_state.clone();
        let current = *temp.value();
        temp.set_value(current + 1);
    })).frame(120.0, 70.0);

    let button_disabled = PlainButton::new(|env: &mut Environment, _:_| {}).enabled(false).frame(120.0, 70.0);

    application.set_scene(Window::new(
        "Plain Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        *VStack::new(vec![
            Text::new(counter_text).font_size(40u32),
            button.boxed(),
            button_disabled.boxed(),
        ]).spacing(20.0)
    ).close_application_on_window_close());

    application.launch();
}
