use carbide_core::time::*;
use carbide_core::animation::ease_in_out;
use carbide_core::color::{LIGHT_BLUE, RED};
use carbide_core::draw::{Color, Dimension};
use carbide_core::state::AnimatedState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let sigma = AnimatedState::custom(ease_in_out)
        .duration(Duration::from_secs_f64(2.1))
        .repeat_alternate()
        .range(1.0, 10.0);

    let color = AnimatedState::custom(ease_in_out)
        .duration(Duration::from_secs_f64(0.6))
        .repeat_alternate()
        .range(RED, LIGHT_BLUE);

    let offset_x = AnimatedState::custom(ease_in_out)
        .duration(Duration::from_secs_f64(1.0))
        .repeat_alternate()
        .range(-20, 20);

    let offset_y = AnimatedState::custom(ease_in_out)
        .duration(Duration::from_secs_f64(1.7))
        .repeat_alternate()
        .range(-20, 20);

    application.set_scene(
        Window::new(
            "Shadow example - Carbide",
            Dimension::new(600.0, 600.0),
            VStack::new((
                Text::new("Hello world!").font_size(42),
                Image::new_icon("images/rust.png")
            )).foreground_color(Color::new_rgba(170, 170, 170, 255))
                .shadow(sigma.clone(), color.clone(), offset_x, offset_y)
        )
    );

    application.launch()
}
