#[macro_use]
extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use futures::executor::block_on;
use serde::Deserialize;
use serde::Serialize;

use carbide_controls::PopUpButton;
use carbide_core::color::RED;
use carbide_core::widget::*;
use carbide_core::widget::EnvironmentColor;
use carbide_wgpu::window::Window;

use self::Day::{Friday, Monday, Saturday, Sunday, Thursday, Tuesday, Wednesday};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

    let icon_path = Window::<u32>::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Pop up Button Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
        0,
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    family.add_font(
        "fonts/NotoSans/NotoSans-Italic.ttf",
        FontWeight::Normal,
        FontStyle::Italic,
    );
    family.add_font(
        "fonts/NotoSans/NotoSans-Bold.ttf",
        FontWeight::Bold,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    let selected_index = CommonState::new_local_with_key(&0);

    let selected_model = CommonState::new_local_with_key(&vec![
        Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday,
    ]);

    window.set_widgets(
        PopUpButton::new(Box::new(selected_model), Box::new(selected_index)).frame(120.0, 100.0),
    );

    window.launch();
}
