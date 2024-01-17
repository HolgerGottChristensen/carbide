use carbide_controls::{ControlsExt, TextInput};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let text_state = LocalState::new(Ok(0f64));
    let text_state2 = LocalState::new("Hello world!".to_string());

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
        "Text Input Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            TextInput::new(text_state.clone()).accent_color(EnvironmentColor::Green).boxed(),
            TextInput::new(text_state.clone()).accent_color(EnvironmentColor::Purple).boxed(),
            TextInput::new(text_state2.clone()).boxed(),
            TextInput::new(text_state2.clone()).obscure().boxed(),
            TextInput::new(text_state2.clone()).obscure_with('©').boxed(),
            Empty::new().frame(10.0, 10.0),
            TextInput::new(text_state2.clone()).enabled(false).boxed(),

        ))
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ).close_application_on_window_close());

    application.launch();
}
