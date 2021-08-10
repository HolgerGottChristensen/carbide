use std::f64::consts::PI;
use std::time::Duration;

use carbide_core::draw::Position;
use carbide_core::environment::*;
use carbide_core::prelude::{ease, ease_in, elastic_in, elastic_in_out};
use carbide_core::state::{
    AnimatedState, bounce_in, bounce_in_out, bounce_out, cubic_bezier, ease_in_out, ease_in_to_linear,
    ease_out, elastic_out, fast_linear_to_slow_ease_in, fast_out_slow_in, MapOwnedState,
    slow_middle,
};
use carbide_core::text::*;
use carbide_core::widget::*;
use carbide_core::widget::canvas::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "If and else example".to_string(),
        800,
        1200,
        Some(icon_path.clone()),
    );

    let animation_move_x_state1 = AnimatedState::linear(Duration::new(2, 0), window.environment())
        .repeat_alternate()
        .range(0.0, 300.0);

    let animation_move_x_state2 = MapOwnedState::new(
        AnimatedState::custom(
            Duration::new(2, 0),
            fast_linear_to_slow_ease_in,
            window.environment(),
        )
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state3 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), ease_in, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state4 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), ease_out, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state5 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), ease_in_out, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state6 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), fast_out_slow_in, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state7 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), slow_middle, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state8 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), elastic_in, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state9 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), elastic_out, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state10 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), elastic_in_out, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state11 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), bounce_in, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state12 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), bounce_out, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    let animation_move_x_state13 = MapOwnedState::new(
        AnimatedState::custom(Duration::new(2, 0), bounce_in_out, window.environment())
            .repeat_alternate(),
        |percent: &f64| *percent * 300.0,
    );

    window.set_widgets(
        VStack::new(vec![
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state1, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state2, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state3, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state4, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state5, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state6, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state7, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state8, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state9, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state10, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state11, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state12, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
            HStack::new(vec![
                Circle::new()
                    .fill(EnvironmentColor::Accent)
                    .stroke(EnvironmentColor::Label)
                    .frame(30, 30)
                    .offset(animation_move_x_state13, 0.0),
                Spacer::new(SpacerDirection::Horizontal),
            ]),
        ])
            .spacing(10.0)
            .padding(30.0),
    );

    /*

        IfElse::new(animation_state)
            .when_true(Rectangle::new(vec![]).fill(EnvironmentColor::Orange))
            .when_false(Rectangle::new(vec![]).fill(EnvironmentColor::Green))
            .offset(animation_move_x_state, 0.0)
    */

    window.run_event_loop();
}
