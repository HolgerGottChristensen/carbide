use std::time::Duration;

use carbide_core::animation::ease_in_out;
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::AnimatedState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let position_x = AnimatedState::custom(ease_in_out)
        .duration(Duration::new(5, 0))
        .repeat_alternate()
        .range(-180.0, 180.0);

    application.set_scene(Window::new(
        "Filter example - Carbide",
        Dimension::new(600.0, 450.0),
        ZStack::new((
            Image::new("images/landscape.png")
                .scaled_to_fill()
                .frame(500.0, 400.0),
            Filter::new(ImageFilter::sobel(), Empty::new())
                .clip_shape(Circle::new())
                .frame(200.0, 200.0)
                .offset(position_x.clone(), 0.0),
            Circle::new()
                .stroke(EnvironmentColor::Accent)
                .stroke_style(1.0)
                .frame(200.0, 200.0)
                .offset(position_x, 0.0),
        ))
    ));

    application.launch();
}
