use carbide::time::*;
use carbide::animation::ease_in_out;
use carbide::draw::Dimension;
use carbide::state::AnimatedState;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

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
