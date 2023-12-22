use carbide_controls::RadioButton;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
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
    let shape_state2 = LocalState::new(Shape::Rectangle);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Radio Button Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            RadioButton::new("Rectangle", Shape::Rectangle, shape_state.clone()),
            RadioButton::new("Circle", Shape::Circle, shape_state.clone()),
            RadioButton::new("Triangle", Shape::Triangle, shape_state.clone()),
            RadioButton::new("Star", Shape::Star, shape_state.clone()),
            Empty::new().frame(10.0, 10.0),
            RadioButton::new("Checked - Disabled", Shape::Rectangle, shape_state2.clone()).enabled(false),
            RadioButton::new("Unchecked - Disabled", Shape::Circle, shape_state2.clone()).enabled(false),
        ))
            .spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .accent_color(EnvironmentColor::Orange)
            .padding(EdgeInsets::all(40.0)),
    ).close_application_on_window_close());

    application.launch();
}
