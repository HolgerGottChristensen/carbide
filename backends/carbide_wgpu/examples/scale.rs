use carbide_core::time::*;
use carbide_core::animation::ease_in_out;
use carbide_core::draw::Dimension;
use carbide_core::state::AnimatedState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let scale = AnimatedState::custom(ease_in_out)
        .duration(Duration::new(5, 0))
        .repeat_alternate()
        .range(0.001, 2.0);

    application.set_scene(Window::new(
        "Scale example - Carbide",
        Dimension::new(600.0, 600.0),
        Image::new("images/landscape.png")
            .scaled_to_fill()
            .clip_shape(Rectangle::new())
            .scale_effect(scale)
            .frame(200.0, 200.0)
            .border(),
    ));

    application.launch();
}
