use std::f64::consts::PI;

use carbide_core::draw::{Dimension, Position};
use carbide_core::draw::image::ImageId;
use carbide_core::environment::*;
use carbide_core::locate_folder;
use carbide_core::widget::canvas::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Frame example",
            Dimension::new(600.0, 600.0),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 100.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
