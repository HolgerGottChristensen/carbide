use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Frame example - Carbide",
            Dimension::new(300.0, 300.0),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 100.0)
        )
    );

    application.launch()
}
