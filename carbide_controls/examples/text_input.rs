use carbide_controls::TextInput;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let text_state = LocalState::new(Ok(0i128));
    let text_state2 = LocalState::new("Hello world!".to_string());

    let mut application = Application::new()
        .with_asset_fonts();

    /* // Load emoji font on macos
    let mut family = FontFamily::new("Apple Color Emoji");
    family.add_bitmap_font_with_hints(
        "/System/Library/Fonts/Apple Color Emoji.ttc",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    application.add_font_family(family);*/

    application.set_scene(Window::new(
        "Text Input Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            TextInput::new(text_state.clone()).accent_color(EnvironmentColor::Green),
            TextInput::new(text_state.clone()).accent_color(EnvironmentColor::Purple),
            TextInput::new(text_state2.clone()),
            TextInput::new(text_state2.clone()).obscure(),
            TextInput::new(text_state2.clone()).obscure_with_char('©'),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ).close_application_on_window_close());

    application.launch();
}
