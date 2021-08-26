extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::{CheckBox, CheckBoxValue};
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Checkbox Example - Carbide".to_string(),
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

    let checkbox_state1 = CommonState::new_local_with_key(&CheckBoxValue::False);
    let checkbox_state2 = CommonState::new_local_with_key(&CheckBoxValue::Intermediate);
    let checkbox_state3 = CommonState::new_local_with_key(&CheckBoxValue::True);
    let checkbox_state4 = CommonState::new_local_with_key(&CheckBoxValue::False);

    window.set_widgets(
        VStack::new(vec![
            CheckBox::new("Rectangle", checkbox_state1).frame(100.0, 26.0),
            CheckBox::new("Circle", checkbox_state2).frame(100.0, 26.0),
            CheckBox::new("Triangle", checkbox_state3).frame(100.0, 26.0),
            CheckBox::new("Star", checkbox_state4).frame(100.0, 26.0),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    );

    window.launch();
}
