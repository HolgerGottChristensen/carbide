extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use futures::executor::block_on;
use serde::{Deserialize, Serialize};

use carbide_controls::PlainRadioButton;
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum Shape {
    Circle,
    Rectangle,
    Triangle,
    Star,
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Rectangle
    }
}

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Plain Radio Button Example - Carbide".to_string(), 800, 1200, Some(icon_path), String::from("Hejsa"));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    let shape_state = CommonState::new_local_with_key(&Shape::Rectangle);

    window.set_widgets(
        SharedState::new(shape_state.clone(),
         VStack::initialize(vec![
             PlainRadioButton::new("Rectangle", Shape::Rectangle, shape_state.clone())
                 .border(),
             PlainRadioButton::new("Circle", Shape::Circle, shape_state.clone())
                 .border(),
             PlainRadioButton::new("Triangle", Shape::Triangle, shape_state.clone())
                 .border(),
             PlainRadioButton::new("Star", Shape::Star, shape_state.clone())
                 .border(),
         ]).spacing(10.0).padding(EdgeInsets::all(40.0)))
    );

    window.run_event_loop();

}