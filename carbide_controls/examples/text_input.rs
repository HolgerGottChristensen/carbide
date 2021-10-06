extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::{PlainSwitch, PlainTextInput, TextInput};
use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Text Input Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    let mut family = FontFamily::new("Apple Color Emoji");
    family.add_bitmap_font_with_hints(
        "/System/Library/Fonts/Apple Color Emoji.ttc",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    let text_state = LocalState::new("Hello World!".to_string());

    window.set_widgets(
        VStack::new(vec![
            TextInput::new(text_state),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    );

    window.launch();
}
