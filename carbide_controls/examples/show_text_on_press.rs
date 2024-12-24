use carbide_controls::{PASSWORD_CHAR, PlainTextInput, ControlsExt};
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

    application.set_scene(Window::new(
        "Show/hide text Example - Carbide",
        Dimension::new(300.0, 600.0),
        VStack::new((
            PlainTextInput::new(text_state.clone())
                .obscure(obscure)
                .font_size(EnvironmentFontSize::Title)
                .border(),
            Button::new("Show/hide", closure!(|_|{}))
                .pressed(pressed)
                .button_style(BorderedProminentStyle)
        ))
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ));

    application.launch();
}
