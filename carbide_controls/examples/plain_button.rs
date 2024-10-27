use carbide_controls::{PlainButton};
use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, State, ReadStateExtNew};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let counter_state = LocalState::new(0);

    let counter_text = counter_state
        .map(|count: &i32| format!("Count: {}", count));

    let button = PlainButton::new(closure!(|_| {
        *$counter_state = *$counter_state + 1;
    })).frame(120.0, 50.0);

    let button_disabled = PlainButton::new(|_| {}).enabled(false).frame(120.0, 50.0);

    application.set_scene(Window::new(
        "Plain Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Text::new(counter_text).font_size(40u32),
            button,
            button_disabled,
        )).spacing(20.0)
    ).close_application_on_window_close());

    application.launch();
}
