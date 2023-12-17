use std::time::Duration;
use carbide_core::draw::Dimension;

use carbide_core::state::AnimatedState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new();

    let rotation = AnimatedState::linear(Some(application.environment()))
        .duration(Duration::new(5, 0))
        .repeat_alternate()
        .range(0.0, 180.0);

    application.set_scene(Window::new(
        "Rotation 3d example",
        Dimension::new(400.0, 600.0),
        Image::new("images/landscape.png")
            .scaled_to_fill()
            .clip_shape(Rectangle::new())
            .rotation_3d_effect(rotation, 0.0)
            .with_fov(1.0)
            .frame(200.0, 100.0)
            .border()
    ).close_application_on_window_close());

    application.launch()
}
