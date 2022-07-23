use carbide_core::draw::Dimension;
use carbide_core::prelude::{EnvironmentColor, Rectangle};
use carbide_core::widget::ZStack;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new("Hello multiple windows", Dimension::new(300.0, 200.0),ZStack::new(vec![
            Rectangle::new(),
            //Window::new("Hello from window 2", Dimension::new(300.0, 100.0), Rectangle::new().fill(EnvironmentColor::Red)),
            //Window::new("Hello from window 3", Dimension::new(300.0, 100.0), Rectangle::new().fill(EnvironmentColor::Green)),
            //Window::new("Hello from window 4", Dimension::new(300.0, 100.0), Rectangle::new().fill(EnvironmentColor::Yellow)),
        ]))
    );

    application.launch()
}