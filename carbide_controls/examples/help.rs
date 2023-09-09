use carbide_controls::{ControlsExt, TextInput};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let text_state = LocalState::new("Hello world!".to_string());
    let text_state2 = LocalState::new("Hej verden!".to_string());

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
        "Help example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            TextInput::new(text_state.clone())
                .help("This is a help")
                .boxed(),
            TextInput::new(text_state2.clone())
                .help("This is a help\n on multiple lines")
                .boxed(),
            Empty::new().frame(10.0, 400.0)
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ).close_application_on_window_close());

    application.launch();
}
