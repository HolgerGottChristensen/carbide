extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::{PlainSwitch, Switch};
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
        "Switch Example - Carbide".to_string(),
        400,
        600,
        Some(icon_path),
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    let switch_state1 = LocalState::new(false);
    let switch_state2 = LocalState::new(true);
    let switch_state3 = LocalState::new(true);
    let switch_state4 = LocalState::new(false);

    window.set_widgets(
        VStack::new(vec![
            Switch::new("Rectangle", switch_state1),
            Switch::new("Circle", switch_state2),
            Switch::new("Triangle", switch_state3),
            Switch::new("Star", switch_state4),
        ])
            .spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .accent_color(EnvironmentColor::Purple),
    );

    window.launch();
}
