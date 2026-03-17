use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "Image SVG example - Carbide",
        Dimension::new(800.0, 600.0),
        Image::new("icons/ambulance.svg")
            .color(EnvironmentColor::Label)
            .border(),
    ));

    application.launch();
}
