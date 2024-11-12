use carbide_controls::{PASSWORD_CHAR, PlainTextInput};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentFontSize;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let text_state = LocalState::new("Hello World!".to_string());
    //let text_state = LocalState::new("ধারা ১ সমস্ত মানুষ".to_string());
    //let text_state = LocalState::new("👨‍👨‍👧‍👧".to_string());

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Plain Text Input Example - Carbide",
        Dimension::new(300.0, 600.0),
        VStack::new((
            PlainTextInput::new(text_state.clone())
                .font_size(EnvironmentFontSize::Title)
                .border(),
            PlainTextInput::new(text_state.clone())
                .obscure(Some(PASSWORD_CHAR))
                .font_size(EnvironmentFontSize::Title)
                .border()
        ))
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ));

    application.launch();
}
