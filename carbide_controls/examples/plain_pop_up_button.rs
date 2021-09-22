#[macro_use]
extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;


use carbide_controls::PlainPopUpButton;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

use crate::Day::{Friday, Monday, Saturday, Sunday, Thursday, Tuesday, Wednesday};

#[derive(Debug, Clone, PartialEq)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Default for Day {
    fn default() -> Self {
        Monday
    }
}

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Plain Pop up Button Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let selected = LocalState::new(Monday);

    let model = LocalState::new(vec![
        Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday,
    ]);

    window.set_widgets(
        PlainPopUpButton::new(model, selected)
            .border()
            .color(EnvironmentColor::Red)
            .clip()
            .frame(120.0, 40.0),
    );

    window.launch();
}
