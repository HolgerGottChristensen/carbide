extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::PlainSwitch;
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Plain Switch Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
        String::from("Hejsa"),
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

    let switch_state1 = CommonState::new_local_with_key(&false);
    let switch_state2 = CommonState::new_local_with_key(&true);
    let switch_state3 = CommonState::new_local_with_key(&true);
    let switch_state4 = CommonState::new_local_with_key(&false);

    window.set_widgets(
        VStack::new(vec![
            PlainSwitch::new("Rectangle", switch_state1).border(),
            PlainSwitch::new("Circle", switch_state2).border(),
            PlainSwitch::new("Triangle", switch_state3).border(),
            PlainSwitch::new("Star", switch_state4).border(),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    );

    window.run_event_loop();
}
