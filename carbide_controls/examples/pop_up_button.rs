use carbide_controls::PopUpButton;
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

use crate::Month::{
    April, December, February, January, July, June, March, May, November, October, September,
};

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
    let selected = LocalState::new(January);

    let model = LocalState::new(vec![
        January, February, March, April, May, June, July, September, October, November, December,
    ]);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Pop up Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        PopUpButton::new(model, selected).frame_fixed_width(200)
    ).close_application_on_window_close());

    application.launch();
}
