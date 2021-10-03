extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use futures::executor::block_on;
use serde::{Deserialize, Serialize};

use carbide_controls::{PlainRadioButton, RadioButton};
use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

#[derive(Clone, Debug, PartialEq)]
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

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Radio Button Example - Carbide".to_string(),
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

    let shape_state = LocalState::new(Shape::Rectangle);

    window.set_widgets(
        VStack::new(vec![
            RadioButton::new("Rectangle", Shape::Rectangle, shape_state.clone()),
            RadioButton::new("Circle", Shape::Circle, shape_state.clone()),
            RadioButton::new("Triangle", Shape::Triangle, shape_state.clone()),
            RadioButton::new("Star", Shape::Star, shape_state.clone()),
        ]).spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .accent_color(EnvironmentColor::Orange)
            .padding(EdgeInsets::all(40.0)),
    );

    window.launch();
}
