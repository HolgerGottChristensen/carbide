use carbide_controls::{PASSWORD_CHAR, PlainTextInput, ControlsExt, TextInput};
use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentFontSize;
use carbide_core::state::{LocalState, ReadStateExtNew};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let text_state = LocalState::new("Hello World!".to_string());

    let mut application = Application::new()
        .with_asset_fonts();

    let pressed = LocalState::new(false);

    let obscure = pressed.map(|pressed| {
        if *pressed { None } else { Some(PASSWORD_CHAR) }
    });

    let icon = IfElse::new(pressed.clone())
        .when_false(Image::system("eye-off").resizeable())
        .when_true(Image::system("eye").resizeable());

    application.set_scene(Window::new(
        "Show/hide text Example - Carbide",
        Dimension::new(300.0, 100.0),
        HStack::new((
            TextInput::new(text_state.clone())
                .obscure_with(obscure),
            Button::new(icon.padding(3.0), closure!(|_|{}))
                .pressed(pressed)
                .button_style(BorderedProminentStyle)
                .aspect_ratio(Dimension::new(1.0, 1.0))
        ))
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ));

    application.launch();
}
