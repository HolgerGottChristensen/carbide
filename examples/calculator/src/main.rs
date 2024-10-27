use std::ops::Deref;

use carbide::{closure, Application, Window};
use carbide::color::ColorExt;
use carbide::controls::PlainButton;
use carbide::draw::{Color, Dimension};
use carbide::environment::{EnvironmentColor, IntoColorReadState};
use carbide::state::{LocalState, Map3, ReadState, State, ReadStateExtNew};
use carbide::widget::*;

use crate::calculator_state::{CalculatorState, Operation};

pub mod calculator_state;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let mut calculator_state = LocalState::new(CalculatorState::new());

    let upper_display_string = calculator_state.map(|m: &CalculatorState| m.get_upper_display());
    let display_string = calculator_state.map(|m: &CalculatorState| m.get_display());

    let display = HStack::new((
        Spacer::new(),
        VStack::new((
            Text::new(upper_display_string).font_size(16),
            Text::new(display_string).font_size(40),
        ))
        .cross_axis_alignment(CrossAxisAlignment::End)
        .spacing(3.0),
    ))
    .frame(0.0, 60.0)
    .expand_width()
    .padding(5.0)
        .boxed();

    application.set_scene(Window::new(
        "My first calculator",
        Dimension::new(235.0, 300.0),
        VStack::new((
            display,
            HStack::new((
                calculator_button(
                    Text::new("AC").font_size(32),
                    closure!(|_, _| ($calculator_state).clear_all()),
                )
                    .accent_color(EnvironmentColor::TertiarySystemFill),
                calculator_button(
                    Text::new("±").font_size(32),
                    closure!(|_, _| ($calculator_state).negate_current()),
                )
                    .accent_color(EnvironmentColor::TertiarySystemFill),
                calculator_button(
                    Text::new("%").font_size(32),
                    closure!(|_, _| ($calculator_state).percent_to_decimal()),
                )
                    .accent_color(EnvironmentColor::TertiarySystemFill),
                operator_button(Operation::Div, calculator_state.clone()),
            ))
                .spacing(1.0),
            HStack::new((
                number_button(7, calculator_state.clone()),
                number_button(8, calculator_state.clone()),
                number_button(9, calculator_state.clone()),
                operator_button(Operation::Mul, calculator_state.clone()),
            ))
                .spacing(1.0),
            HStack::new((
                number_button(4, calculator_state.clone()),
                number_button(5, calculator_state.clone()),
                number_button(6, calculator_state.clone()),
                operator_button(Operation::Sub, calculator_state.clone()),
            ))
                .spacing(1.0),
            HStack::new((
                number_button(1, calculator_state.clone()),
                number_button(2, calculator_state.clone()),
                number_button(3, calculator_state.clone()),
                operator_button(Operation::Add, calculator_state.clone()),
            ))
                .spacing(1.0),
            HStack::new((
                number_button(0, calculator_state.clone()),
                HStack::new((
                    calculator_button(
                        Text::new(",").font_size(32),
                        closure!(|_, _| ($calculator_state).push_separator()),
                    )
                        .accent_color(EnvironmentColor::SystemFill),
                    operator_button(Operation::Eq, calculator_state.clone()),
                ))
                    .spacing(1.0),
            ))
                .spacing(1.0),
        ))
            .spacing(1.0)
    ).close_application_on_window_close());

    application.launch();
}

fn calculator_button(label: impl Widget, action: impl MouseAreaAction + Clone + 'static) -> impl Widget + WidgetExt {
    let pressed_state = LocalState::new(false);
    let hovered_state = LocalState::new(false);

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
    );

    PlainButton::new(action)
        .delegate(move |_, _, _, _| {
            ZStack::new((
                Rectangle::new().fill(background_color.clone()),
                label.clone(),
            )).boxed()
        })
        .pressed(pressed_state)
        .hovered(hovered_state)
}

fn number_button(number: i64, state: impl State<T=CalculatorState>) -> impl Widget + WidgetExt {
    calculator_button(
        Text::new(number).font_size(32),
        closure!(|_, _| ($state).append(number)),
    )
    .accent_color(EnvironmentColor::SystemFill)
}

fn operator_button(operator: Operation, state: impl State<T=CalculatorState>) -> impl Widget + WidgetExt {
    calculator_button(
        Text::new(operator.to_symbol()).font_size(32),
        closure!(|_, _| ($state).set_operation(operator)),
    )
}
