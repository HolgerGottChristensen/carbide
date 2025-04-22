use log::info;
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    info!("Set scene");

    application.set_scene(
        Window::new(
            "Offset example - Carbide",
            Dimension::new(200.0, 200.0),
            /*Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 100.0)
                .offset(50.0, 50.0)
                .border()*/
            Text::new("Hello world!")
        )
    );

    info!("Take off");

    application.launch()
}
