use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "OnKey example - Carbide",
            Dimension::new(200.0, 200.0),
            Rectangle::new()
                .on_key_pressed(|key, _, _| {
                    println!("Pressed: {:?}", key);
                })
                .on_key_released(|key, _, _| {
                    println!("Released: {:?}", key);
                })
                .padding(10.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
