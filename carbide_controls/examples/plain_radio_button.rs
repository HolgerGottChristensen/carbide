use carbide_controls::PlainRadioButton;
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

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
    let shape_state = LocalState::new(Shape::Rectangle);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Plain Radio Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            PlainRadioButton::new("Rectangle", Shape::Rectangle, shape_state.clone()).border(),
            PlainRadioButton::new("Circle", Shape::Circle, shape_state.clone()).border(),
            PlainRadioButton::new("Triangle", Shape::Triangle, shape_state.clone()).border(),
            PlainRadioButton::new("Star", Shape::Star, shape_state.clone()).border(),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0))
    ).close_application_on_window_close());

    application.launch();
}
