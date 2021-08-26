use std::f64::consts::PI;
use std::time::Duration;

use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::*;
use carbide_core::prelude::{
    Deref, DerefMut, ease, ease_in, elastic_in, elastic_in_out, Flags, Uuid,
};
use carbide_core::state::{
    AnimatedState, bounce_in, bounce_in_out, bounce_out, cubic_bezier, ease_in_out, ease_in_to_linear,
    ease_out, elastic_out, F64State, fast_linear_to_slow_ease_in, fast_out_slow_in, linear,
    MapOwnedState, slow_middle, TState,
};
use carbide_core::text::*;
use carbide_core::widget::*;
use carbide_core::widget::canvas::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();
    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Ball animation example".to_string(),
        800,
        1200,
        Some(icon_path.clone()),
    );

    window.set_widgets(
        VStack::new(vec![
            animation_ball(linear, &window),
            animation_ball(fast_linear_to_slow_ease_in, &window),
            animation_ball(ease_in, &window),
            animation_ball(ease_out, &window),
            animation_ball(ease_in_out, &window),
            animation_ball(fast_out_slow_in, &window),
            animation_ball(slow_middle, &window),
            animation_ball(elastic_in, &window),
            animation_ball(elastic_out, &window),
            animation_ball(elastic_in_out, &window),
            animation_ball(bounce_in, &window),
            animation_ball(bounce_out, &window),
            animation_ball(bounce_in_out, &window),
        ])
            .spacing(10.0)
            .padding(30.0),
    );
    window.launch();
}

fn animation_position_state(curve: fn(f64) -> f64, window: &Window) -> TState<f64> {
    AnimatedState::custom(curve, window.environment())
        .duration(Duration::new(2, 0))
        .repeat_alternate()
        .range(0.0, 300.0)
}

fn animation_ball(curve: fn(f64) -> f64, window: &Window) -> Box<dyn Widget> {
    let state = animation_position_state(curve, window);
    HStack::new(vec![
        Circle::new()
            .fill(EnvironmentColor::Accent)
            .stroke(EnvironmentColor::Label)
            .frame(30, 30)
            .offset(state, 0.0),
        Spacer::new(SpacerDirection::Horizontal),
    ])
}
