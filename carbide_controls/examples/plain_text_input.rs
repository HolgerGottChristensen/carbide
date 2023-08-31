use carbide_controls::{PASSWORD_CHAR, PlainTextInput};
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::{LocalState, TState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let text_state = LocalState::new("Hello World!".to_string());

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Plain Text Input Example - Carbide",
        Dimension::new(200.0, 600.0),
        VStack::new(vec![
            PlainTextInput::new(text_state.clone())
                //.font_size(EnvironmentFontSize::Title)
                .border()
                .boxed()
                //.color(EnvironmentColor::DarkText)
                //.background(Rectangle::new().fill(EnvironmentColor::Blue)),

            /*PlainTextInput::new(text_state)
                .font_size(EnvironmentFontSize::Title)
                .obscure(PASSWORD_CHAR)
                .border()
                .color(EnvironmentColor::DarkText)
                .background(Rectangle::new().fill(EnvironmentColor::Purple)),*/
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ).close_application_on_window_close());

    application.launch();
}
