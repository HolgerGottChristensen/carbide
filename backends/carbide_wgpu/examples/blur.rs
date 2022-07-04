use std::time::Duration;

use carbide_core::environment::*;
use carbide_core::state::{ease_in_out, AnimatedState};
use carbide_core::text::*;
use carbide_core::widget::canvas::*;
use carbide_core::widget::*;
use carbide_core::Color;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Blur example".to_string(),
        600,
        450,
        Some(icon_path.clone()),
    );

    let image_id = window.add_image_from_path("images/landscape.png");

    let position_x = AnimatedState::custom(ease_in_out, Some(window.environment()))
        .duration(Duration::new(5, 0))
        .repeat_alternate()
        .range(-180.0, 180.0);

    let position_neg_x = AnimatedState::custom(ease_in_out, Some(window.environment()))
        .duration(Duration::new(7, 0))
        .repeat_alternate()
        .range(180.0, -180.0);

    window.set_widgets(ZStack::new(vec![
        Image::new(image_id)
            .scaled_to_fill()
            .clip_shape(Rectangle::new())
            .frame(500.0, 400.0),
        Blur::gaussian(10.0)
            .frame(200.0, 200.0)
            .offset(position_x.clone(), 0.0),
        Blur::mean(3)
            .clip_shape(Circle::new())
            .frame(100.0, 100.0)
            .offset(position_neg_x.clone(), 0.0),
        Rectangle::new()
            .stroke(EnvironmentColor::Accent)
            .stroke_style(1.0)
            .frame(200.0, 200.0)
            .offset(position_x, 0.0),
        Circle::new()
            .stroke(EnvironmentColor::Accent)
            .stroke_style(1.0)
            .frame(100.0, 100.0)
            .offset(position_neg_x, 0.0),
    ]));

    window.launch();
}
