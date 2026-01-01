use carbide_controls::slider::Slider;
use carbide_core::color::Color;
use carbide_core::draw::{Dimension};
use carbide_core::state::{LocalState, Map1};
use carbide_core::widget::{Rectangle, Text, VStack, WidgetExt};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let temperature = LocalState::new(4000.0);

    let color = Map1::read_map(temperature.clone(), |temp| {
        Color::temperature(*temp)
    });

    application.set_scene(
        Window::new(
            "Color temperature example - Carbide",
            Dimension::new(600.0, 600.0),
            VStack::new((
                Rectangle::new().fill(color),
                Text::new(Map1::read_map(temperature.clone(), |temp| format!("Color temperature: {}", f64::trunc(*temp)))),
                Slider::new(temperature, 1000.0, 40000.0),
            )).padding(50.0),
        )
    );

    application.launch()
}