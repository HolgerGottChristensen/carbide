use std::time::Duration;

use carbide_core::animation::ease_in_out;
use carbide_core::draw::Dimension;
use carbide_core::state::AnimatedState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let rotation = AnimatedState::custom(ease_in_out, Some(application.environment()))
        .duration(Duration::new(5, 0))
        .repeat_alternate()
        .range(0.0, 360.0);

    application.set_scene(Window::new(
        "Rotation example",
        Dimension::new(400.0, 600.0),
        Image::new("images/landscape.png")
            .scaled_to_fill()
            .clip_shape(Rectangle::new())
            .rotation_effect(rotation)
            .frame(200.0, 200.0)
            .border(),
    ).close_application_on_window_close());

    application.launch();
}
