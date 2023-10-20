use std::ops::Deref;

use carbide::{Application, Window};
use carbide::draw::Color;
use carbide::draw::Dimension;
use carbide::environment::{Environment, EnvironmentColor};
use carbide::state::{LocalState, Map3, ReadState, State, StateExt, TState};
use carbide::widget::*;
use carbide::widget::WidgetExt;
use carbide_controls::{capture, PlainButton};
use carbide_core::state::ReadStateExtNew;

use crate::calculator_state::{CalculatorState, Operation};

pub mod calculator_state;

fn main() {
    env_logger::init();

    let mut application = Application::new()
        .with_asset_fonts();

    let mut calculator_state = LocalState::new(CalculatorState::new());

    let upper_display_string = calculator_state.map(|m: &CalculatorState| m.get_upper_display());
    let display_string = calculator_state.map(|m: &CalculatorState| m.get_display());

    let display = HStack::new(vec![
        Spacer::new(),
        VStack::new(vec![
            Text::new(upper_display_string).font_size(16),
            Text::new(display_string).font_size(40),
        ])
        .cross_axis_alignment(CrossAxisAlignment::End)
        .spacing(3.0),
    ])
    .frame(0.0, 60.0)
    .expand_width()
    .padding(5.0)
        .boxed();

    application.set_scene(Window::new(
        "My first calculator",
        Dimension::new(235.0, 300.0),
        *VStack::new(vec![
            display,
            HStack::new(vec![
                calculator_button(
                    Text::new("AC").font_size(32),
                    capture!([calculator_state], |env: &mut Environment| {
                        calculator_state.value_mut().clear_all();
                    }),
                )
                    .accent_color(EnvironmentColor::TertiarySystemFill)
                    .boxed(),
                calculator_button(
                    Text::new("±").font_size(32),
                    capture!([calculator_state], |env: &mut Environment| {
                        calculator_state.value_mut().negate_current();
                    }),
                )
                    .accent_color(EnvironmentColor::TertiarySystemFill)
                    .boxed(),
                calculator_button(
                    Text::new("%").font_size(32),
                    capture!([calculator_state], |env: &mut Environment| {
                        calculator_state.value_mut().percent_to_decimal();
                    }),
                )
                    .accent_color(EnvironmentColor::TertiarySystemFill)
                    .boxed(),
                operator_button(Operation::Div, &calculator_state),
            ])
                .spacing(1.0),
            HStack::new(vec![
                number_button(7, &calculator_state),
                number_button(8, &calculator_state),
                number_button(9, &calculator_state),
                operator_button(Operation::Mul, &calculator_state),
            ])
                .spacing(1.0),
            HStack::new(vec![
                number_button(4, &calculator_state),
                number_button(5, &calculator_state),
                number_button(6, &calculator_state),
                operator_button(Operation::Sub, &calculator_state),
            ])
                .spacing(1.0),
            HStack::new(vec![
                number_button(1, &calculator_state),
                number_button(2, &calculator_state),
                number_button(3, &calculator_state),
                operator_button(Operation::Add, &calculator_state),
            ])
                .spacing(1.0),
            HStack::new(vec![
                number_button(0, &calculator_state),
                HStack::new(vec![
                    calculator_button(
                        Text::new(",").font_size(32),
                        capture!([calculator_state], |env: &mut Environment| {
                            calculator_state.value_mut().push_separator();
                        }),
                    )
                        .accent_color(EnvironmentColor::SystemFill)
                        .boxed(),
                    operator_button(Operation::Eq, &calculator_state),
                ])
                    .spacing(1.0),
            ])
                .spacing(1.0),
        ])
            .spacing(1.0)
    ).close_application_on_window_close());

    application.launch();
}

fn calculator_button(label: Box<dyn Widget>, action: impl Action + Clone + 'static) -> Box<dyn Widget> {
    let pressed_state: TState<bool> = LocalState::new(false).into();
    let hovered_state: TState<bool> = LocalState::new(false).into();

    let background_color = Map3::read_map(
        pressed_state.clone(),
        hovered_state.clone(),
        EnvironmentColor::Accent.color(),
        |pressed: &bool, hovered: &bool, base_color: &Color| {
            if *pressed {
                base_color.darkened(0.05)
            } else if *hovered {
                base_color.lightened(0.05)
            } else {
                *base_color
            }
        },
    )
    .ignore_writes();

    PlainButton::new(action)
        .delegate(move |_, _, _, _| {
            ZStack::new(vec![
                Rectangle::new().fill(background_color.clone()),
                label.clone(),
            ]).boxed()
        })
        .pressed(pressed_state)
        .hovered(hovered_state)
        .boxed()
}

fn number_button(number: i64, state: &TState<CalculatorState>) -> Box<dyn Widget> {
    calculator_button(
        Text::new(number).font_size(32),
        capture!([state], |env: &mut Environment| {
            state.value_mut().append(number);
        }),
    )
    .accent_color(EnvironmentColor::SystemFill)
        .boxed()
}

fn operator_button(operator: Operation, state: &TState<CalculatorState>) -> Box<dyn Widget> {
    calculator_button(
        Text::new(operator.to_symbol()).font_size(32),
        capture!([state], |env: &mut Environment| {
            state.value_mut().set_operation(operator);
        }),
    )
}
