use std::ops::Deref;
use env_logger::Env;
use carbide_controls::{Action, capture, PlainButton};
use carbide_core::environment::{Environment, EnvironmentColor};
use carbide_core::widget::*;
use carbide_wgpu::window::*;

use carbide_core::prelude::LocalState;
use carbide_core::state::{BoolState, StateExt, StringState, TState};
use carbide_core::text::FontFamily;
use crate::calculator_state::{CalculatorState, Operation};
use carbide_core::widget::WidgetExt;


pub mod calculator_state;

fn main() {
    env_logger::init();
    let mut window = Window::new(
        "My first calculator",
        470,
        600,
        None,
    );

    let mut family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf",
        "fonts/NotoSans/NotoSans-Italic.ttf",
        "fonts/NotoSans/NotoSans-Bold.ttf",
    ]);
    window.add_font_family(family);


    let mut state = LocalState::new(CalculatorState::new());

    let upper_display_string = state.mapped(|m: &CalculatorState| m.get_upper_display());
    let display_string = state.mapped(|m: &CalculatorState| m.get_display());

    let display = HStack::new(vec![
        Spacer::new(),
        VStack::new(vec![
            Text::new(upper_display_string)
                .font_size(16),
            Text::new(display_string)
                .font_size(32)
        ]).cross_axis_alignment(CrossAxisAlignment::End)
            .spacing(3.0)
    ]).frame(SCALE, 60.0)
        .padding(5);

    window.set_widgets(
        VStack::new(vec![
            display,
            HStack::new(vec![
                calculator_button(Text::new("AC").font_size(32), capture!([state], |env: &mut Environment| {
                        state.clear_all()
                    })).accent_color(EnvironmentColor::TertiarySystemFill),
                calculator_button(Text::new("±").font_size(32), capture!([state], |env: &mut Environment| {
                        state.negate_current()
                    })).accent_color(EnvironmentColor::TertiarySystemFill),
                calculator_button(Text::new("%").font_size(32), capture!([state], |env: &mut Environment| {
                        state.percent_to_decimal()
                    })).accent_color(EnvironmentColor::TertiarySystemFill),
                operator_button(Operation::Div, &state),
            ]).spacing(1.0),
            HStack::new(vec![
                number_button(7, &state),
                number_button(8, &state),
                number_button(9, &state),
                operator_button(Operation::Mul, &state),
            ]).spacing(1.0),
            HStack::new(vec![
                number_button(4, &state),
                number_button(5, &state),
                number_button(6, &state),
                operator_button(Operation::Sub, &state),
            ]).spacing(1.0),
            HStack::new(vec![
                number_button(1, &state),
                number_button(2, &state),
                number_button(3, &state),
                operator_button(Operation::Add, &state),
            ]).spacing(1.0),
            HStack::new(vec![
                number_button(0, &state),
                HStack::new(vec![
                    calculator_button(Text::new(",").font_size(32), capture!([state], |env: &mut Environment| {
                        state.push_separator()
                    })).accent_color(EnvironmentColor::SystemFill),
                    operator_button(Operation::Eq, &state),
                ]).spacing(1.0),

            ]).spacing(1.0),
        ]).spacing(1.0)
            .accent_color(EnvironmentColor::Orange)
    );

    window.launch();
}

fn calculator_button(label: Box<dyn Widget>, action: impl Action + 'static) -> Box<dyn Widget> {
    let pressed_state: BoolState = LocalState::new(false).into();

    let background_color = pressed_state.mapped_env(move |pressed: &bool, _: &_, env: &Environment| {
        if *pressed {
            env.env_color(EnvironmentColor::Accent).unwrap().darkened(0.05)
        } else {
            env.env_color(EnvironmentColor::Accent).unwrap()
        }
    });

    PlainButton::new(
    ZStack::new(vec![
        Rectangle::new().fill(background_color),
        label
    ])).on_click(action)
        .pressed(pressed_state)
}

fn number_button(number: i64, state: &TState<CalculatorState>) -> Box<dyn Widget> {
    calculator_button(Text::new(number).font_size(32), capture!([state], |env: &mut Environment| {
        state.append(number)
    })).accent_color(EnvironmentColor::SystemFill)
}

fn operator_button(operator: Operation, state: &TState<CalculatorState>) -> Box<dyn Widget> {
    calculator_button(Text::new(operator.to_symbol()).font_size(32), capture!([state], |env: &mut Environment| {
        state.set_operation(operator)
    }))
}
