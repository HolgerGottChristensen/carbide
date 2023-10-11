use std::time::Duration;

use carbide_core::animation::{bounce_in, bounce_in_out, bounce_out, ease_in, ease_in_out, ease_out, elastic_in, elastic_in_out, elastic_out, fast_linear_to_slow_ease_in, fast_out_slow_in, linear, slow_middle};
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::{AnimatedState, ReadState, TState};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new();

    let env = application.environment();

    let widgets = VStack::new(vec![
        animation_ball(linear, env),
        animation_ball(fast_linear_to_slow_ease_in, env),
        animation_ball(ease_in, env),
        animation_ball(ease_out, env),
        animation_ball(ease_in_out, env),
        animation_ball(fast_out_slow_in, env),
        animation_ball(slow_middle, env),
        animation_ball(elastic_in, env),
        animation_ball(elastic_out, env),
        animation_ball(elastic_in_out, env),
        animation_ball(bounce_in, env),
        animation_ball(bounce_out, env),
        animation_ball(bounce_in_out, env),
    ])
        .spacing(10.0)
        .padding(30.0);

    application.set_scene(Window::new(
        "Ball animation example",
        Dimension::new(400.0, 600.0),
        widgets,
    ).close_application_on_window_close());

    application.launch()
}

fn animation_position_state(curve: fn(f64) -> f64, env: &Environment) -> impl ReadState<T=f64> {
    AnimatedState::custom(curve, Some(env))
        .duration(Duration::new(2, 0))
        .repeat_alternate()
        .range(0.0, 300.0)
}

fn animation_ball(curve: fn(f64) -> f64, env: &Environment) -> Box<dyn Widget> {
    let state = animation_position_state(curve, env);
    HStack::new(vec![
        Circle::new()
            .fill(EnvironmentColor::Accent)
            .stroke(EnvironmentColor::Label)
            .frame(30.0, 30.0)
            .offset(state, 0.0),
        Spacer::new(),
    ])
}
