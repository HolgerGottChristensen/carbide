use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "AspectRatio example",
            Dimension::new(200.0, 200.0),
            ZStack::new((
                Rectangle::new().fill(EnvironmentColor::Red),
                Rectangle::new()
                    .aspect_ratio(Dimension::new(16.0, 9.0))
            )).padding(40.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
