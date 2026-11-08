use carbide::controls::TextInput;
use carbide::draw::Dimension;
use carbide::environment::EnvironmentColor;
use carbide::state::{LocalState};
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let text_state = LocalState::new(Ok(0i64));
    let text_state2 = LocalState::new("Hello world!".to_string());

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
        "Text Input Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            TextInput::new(text_state.clone()).accent_color(EnvironmentColor::Green),
            TextInput::new(text_state.clone()).accent_color(EnvironmentColor::Purple),
            TextInput::new(text_state2.clone()),
            TextInput::new(text_state2.clone()).obscure(),
            TextInput::new(text_state2.clone()).obscure_with('©'),
            Empty::new().frame(10.0, 10.0),
            TextInput::new(text_state2.clone()).enabled(false),

        ))
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ));

    application.launch();
}
