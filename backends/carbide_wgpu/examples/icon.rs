use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "Icon example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Image::new_icon("images/rust.png")
                .foreground_color(EnvironmentColor::Accent),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(50.0, 50.0),
        )).accent_color(EnvironmentColor::Green),
    ));

    application.launch();
}
