mod calculator;

use conrod_wgpu::window::Window;
use futures::executor::block_on;
use conrod_core::window::TWindow;
use conrod_core::state::state::State;
use conrod_core::widget::primitive::v_stack::VStack;
use conrod_core::widget::{Text, Image, Rectangle, HStack, SCALE, Oval, Frame};
use conrod_core::widget::complex::SyncTest;
use conrod_core::color::{GREEN, LIGHT_BLUE, RED, DARK_GREEN};
use conrod_core::widget::primitive::widget::WidgetExt;
use conrod_core::widget::primitive::spacer::{Spacer, SpacerDirection};
use conrod_core::widget::primitive::edge_insets::EdgeInsets;
use self::calculator::calculator_button::CalculatorButton;
use calculator::calculator_state::CalculatorState;
use calculator::calculator_state::Operation;
use conrod_core::layout::CrossAxisAlignment;

fn main() {
    env_logger::init();
    let mut window = block_on(Window::new("My first calculator".to_string(), 400, 550, None, CalculatorState::new()));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_press.png").unwrap();
    let rust_image1 = window.add_image("images/rust_hover.png").unwrap();
    let rust_image2 = window.add_image("images/rust.png").unwrap();


    window.set_widgets(
        VStack::initialize(vec![
            Rectangle::initialize(vec![
                HStack::initialize(vec![
                    Spacer::new(SpacerDirection::Horizontal),
                    VStack::initialize(
                    vec![
                            Text::initialize(State::GlobalState {
                            function: |global_state: &CalculatorState| {
                                global_state.get_upper_display()
                            },
                            function_mut: None,
                            latest_value: "0".to_string()
                        }, vec![]).font_size(30.into()),
                            Text::initialize(State::GlobalState {
                            function: |global_state: &CalculatorState| {
                                global_state.get_display()
                            },
                            function_mut: None,
                            latest_value: "0".to_string()
                        }, vec![]).font_size(45.into())
                    ]).cross_axis_alignment(CrossAxisAlignment::End)
                ]).padding(EdgeInsets::all(10.0))
            ])
                .fill(DARK_GREEN)
                .frame(-1.0, 150.0),
            HStack::initialize(vec![

                CalculatorButton::new(
                    Text::initialize("".into(), vec![])
                        .font_size(45.into())
                ).on_released(|b, s| println!("I am clicked")),

                CalculatorButton::new(
                    Text::initialize("".into(), vec![])
                        .font_size(45.into())
                ),
                CalculatorButton::new(
                    Image::new(rust_image, vec![]).resizeable().frame(45.0,45.0)
                ).on_released(|_, s| s.pop_char()),
                CalculatorButton::new(
                    Text::initialize("/".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.set_operation(Operation::Div))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(
                    Text::initialize("7".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.append(7)),
                CalculatorButton::new(
                    Text::initialize("8".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.append(8)),
                CalculatorButton::new(
                    Text::initialize("9".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.append(9)),
                CalculatorButton::new(
                    Text::initialize("×".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.set_operation(Operation::Mul))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(
                    Text::initialize("4".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.append(4)),
                CalculatorButton::new(
                    Text::initialize("5".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.append(5)),
                CalculatorButton::new(
                    Text::initialize("6".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.append(6)),
                CalculatorButton::new(
                    Text::initialize("-".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.set_operation(Operation::Sub))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(
                    Text::initialize("1".into(), vec![])
                        .font_size(45.into())
                ).on_released(|_, s| s.append(1)),
                CalculatorButton::new(Text::initialize("2".into(), vec![]).font_size(45.into()))
                    .on_released(|_, s| s.append(2)),
                CalculatorButton::new(Text::initialize("3".into(), vec![]).font_size(45.into()))
                    .on_released(|_, s| s.append(3)),
                CalculatorButton::new(Text::initialize("+".into(), vec![]).font_size(45.into()))
                    .on_released(|_, s| s.set_operation(Operation::Add))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(Text::initialize("".into(), vec![]).font_size(45.into())),
                CalculatorButton::new(Text::initialize("0".into(), vec![]).font_size(45.into()))
                    .on_released(|_, s| s.append(0)),
                CalculatorButton::new(Text::initialize("".into(), vec![]).font_size(45.into())),
                CalculatorButton::new(Text::initialize("=".into(), vec![]).font_size(45.into()))
                    .on_released(|_, s| s.set_operation(Operation::Eq))
            ]).spacing(3.0),
        ]).spacing(3.0)
    );

    window.run_event_loop();

}