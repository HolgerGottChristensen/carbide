mod calculator;

use carbide_core::widget::*;
use futures::executor::block_on;
use carbide_wgpu::window::Window;
use crate::calculator::calculator_state::{CalculatorState, Operation};
use carbide_core::color::DARK_GREEN;
use crate::calculator::calculator_button::CalculatorButton;


#[macro_use]
extern crate carbide_derive;

fn main() {
    env_logger::init();
    let mut window = block_on(Window::new("My first calculator".to_string(), 400, 550, None, CalculatorState::new()));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_press.png").unwrap();


    window.set_widgets(
        VStack::initialize(vec![
            Rectangle::initialize(vec![
                HStack::initialize(vec![
                    Spacer::new(SpacerDirection::Horizontal),
                    VStack::initialize(
                    vec![
                            Text::initialize(Box::new(CommonState::GlobalState {
                            function: |global_state: &CalculatorState| {
                                global_state.get_upper_display()
                            },
                            function_mut: None,
                            latest_value: "0".to_string()
                        })).font_size(30.into()),
                            Text::initialize(Box::new(CommonState::GlobalState {
                            function: |global_state: &CalculatorState| {
                                global_state.get_display()
                            },
                            function_mut: None,
                            latest_value: "0".to_string()
                        })).font_size(45.into())
                    ]).cross_axis_alignment(CrossAxisAlignment::End)
                ]).padding(EdgeInsets::all(10.0))
            ])
                .fill(DARK_GREEN)
                .frame(SCALE.into(), 150.0.into()),
            HStack::initialize(vec![

                CalculatorButton::new(
                    Text::initialize("".into())
                        .font_size(45.into())
                ).on_released(|_, _| println!("I am clicked")),

                CalculatorButton::new(
                    Text::initialize("".into())
                        .font_size(45.into())
                ),
                CalculatorButton::new(
                    Image::new(rust_image).resizeable().frame(45.0.into(),45.0.into())
                ).on_released(|_, s| s.pop_char()),
                CalculatorButton::new(
                    Text::initialize("/".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.set_operation(Operation::Div))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(
                    Text::initialize("7".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.append(7)),
                CalculatorButton::new(
                    Text::initialize("8".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.append(8)),
                CalculatorButton::new(
                    Text::initialize("9".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.append(9)),
                CalculatorButton::new(
                    Text::initialize("×".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.set_operation(Operation::Mul))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(
                    Text::initialize("4".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.append(4)),
                CalculatorButton::new(
                    Text::initialize("5".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.append(5)),
                CalculatorButton::new(
                    Text::initialize("6".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.append(6)),
                CalculatorButton::new(
                    Text::initialize("-".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.set_operation(Operation::Sub))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(
                    Text::initialize("1".into())
                        .font_size(45.into())
                ).on_released(|_, s| s.append(1)),
                CalculatorButton::new(Text::initialize("2".into()).font_size(45.into()))
                    .on_released(|_, s| s.append(2)),
                CalculatorButton::new(Text::initialize("3".into()).font_size(45.into()))
                    .on_released(|_, s| s.append(3)),
                CalculatorButton::new(Text::initialize("+".into()).font_size(45.into()))
                    .on_released(|_, s| s.set_operation(Operation::Add))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(Text::initialize("".into()).font_size(45.into())),
                CalculatorButton::new(Text::initialize("0".into()).font_size(45.into()))
                    .on_released(|_, s| s.append(0)),
                CalculatorButton::new(Text::initialize("".into()).font_size(45.into())),
                CalculatorButton::new(Text::initialize("=".into()).font_size(45.into()))
                    .on_released(|_, s| s.set_operation(Operation::Eq))
            ]).spacing(3.0),
        ]).spacing(3.0)
    );

    window.run_event_loop();

}