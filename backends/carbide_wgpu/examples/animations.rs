use carbide_core::animation::*;
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::AnimatedState;
use carbide_core::time::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let widgets = VStack::new(vec![
        animation_ball(linear),
        animation_ball(fast_linear_to_slow_ease_in),
        animation_ball(ease_in),
        animation_ball(ease_out),
        animation_ball(ease_in_out),
        animation_ball(fast_out_slow_in),
        animation_ball(slow_middle),
        animation_ball(elastic_in),
        animation_ball(elastic_out),
        animation_ball(elastic_in_out),
        animation_ball(bounce_in),
        animation_ball(bounce_out),
        animation_ball(bounce_in_out),
    ])
        .spacing(10.0)
        .padding(30.0);

    application.set_scene(Window::new(
        "Ball animation example - Carbide",
        Dimension::new(400.0, 600.0),
        widgets,
    ));

    application.launch()
}

fn animation_ball(curve: fn(f64) -> f64) -> impl Widget {
    let state = AnimatedState::custom(curve)
        .duration(Duration::new(2, 0))
        .repeat_alternate()
        .range(-150.0, 150.0);

    Circle::new()
        .fill(EnvironmentColor::Accent)
        .stroke(EnvironmentColor::Label)
        .frame(30.0, 30.0)
        .offset(state, 0.0)
}
