extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::{PlainPopUpButton, PopUpButton};
use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

use crate::Month::{April, December, February, January, July, June, March, May, November, October, September};

#[derive(Debug, Clone, PartialEq)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Default for Month {
    fn default() -> Self {
        January
    }
}

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Pop up Button Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let selected = LocalState::new(January);

    let model = LocalState::new(vec![
        January, February, March, April, May, June, July, September, October, November, December,
    ]);

    window.set_widgets(
        PopUpButton::new(model, selected)
            .frame_expand_height(200),
    );

    window.launch();
}
