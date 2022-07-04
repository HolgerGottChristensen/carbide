use std::f64::consts::PI;
use std::time::Duration;

use carbide_core::draw::Position;
use carbide_core::environment::*;
use carbide_core::layout::BasicLayouter;
use carbide_core::state::{ease_in_out, AnimatedState};
use carbide_core::text::*;
use carbide_core::widget::canvas::*;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Scale example".to_string(),
        600,
        600,
        Some(icon_path.clone()),
    );

    let image_id = window.add_image_from_path("images/landscape.png");

    let scale = AnimatedState::custom(ease_in_out, Some(window.environment()))
        .duration(Duration::new(5, 0))
        .repeat_alternate()
        .range(0.001, 2.0);

    window.set_widgets(
        Image::new(image_id)
            .scaled_to_fill()
            .clip_shape(Rectangle::new())
            .scale_effect(scale)
            .frame(200.0, 200.0)
            .border(),
    );

    window.launch();
}
