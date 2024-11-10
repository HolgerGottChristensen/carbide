use std::time::Duration;

use carbide_core::animation::{bounce_in, bounce_in_out, bounce_out, ease_in, ease_in_out, ease_out, elastic_in, elastic_in_out, elastic_out, fast_linear_to_slow_ease_in, fast_out_slow_in, linear, slow_middle};
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::{AnimatedState, ReadState};
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

fn animation_position_state(curve: fn(f64) -> f64) -> impl ReadState<T=f64> {
    AnimatedState::custom(curve)
        .duration(Duration::new(2, 0))
        .repeat_alternate()
        .range(-150.0, 150.0)
}

fn animation_ball(curve: fn(f64) -> f64) -> impl Widget {
    let state = animation_position_state(curve);

    Circle::new()
        .fill(EnvironmentColor::Accent)
        .stroke(EnvironmentColor::Label)
        .frame(30.0, 30.0)
        .offset(state, 0.0)
}
