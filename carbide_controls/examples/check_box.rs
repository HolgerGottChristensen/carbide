extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use futures::executor::block_on;
use serde::{Deserialize, Serialize};

use carbide_controls::{CheckBox, CheckBoxValue};
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Checkbox Example - Carbide".to_string(), 800, 1200, Some(icon_path), String::from("Hejsa"));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    let checkbox_state1 = CommonState::new_local_with_key(&CheckBoxValue::False);
    let checkbox_state2 = CommonState::new_local_with_key(&CheckBoxValue::Intermediate);
    let checkbox_state3 = CommonState::new_local_with_key(&CheckBoxValue::True);
    let checkbox_state4 = CommonState::new_local_with_key(&CheckBoxValue::False);

    window.set_widgets(
        VStack::initialize(vec![
            CheckBox::new("Rectangle", checkbox_state1)
                .frame(100.0, 26.0),
            CheckBox::new("Circle", checkbox_state2)
                .frame(100.0, 26.0),
            CheckBox::new("Triangle", checkbox_state3)
                .frame(100.0, 26.0),
            CheckBox::new("Star", checkbox_state4)
                .frame(100.0, 26.0),
        ]).spacing(10.0).padding(EdgeInsets::all(40.0))
    );

    window.run_event_loop();

}