mod calculator;

use carbide_wgpu::window::Window;
use futures::executor::block_on;
use carbide_core::window::TWindow;
use carbide_core::state::state::CommonState;
use carbide_core::widget::primitive::v_stack::VStack;
use carbide_core::widget::{Text, Image, Rectangle, HStack, SCALE, Oval, Frame};
use carbide_core::widget::complex::SyncTest;
use carbide_core::color::{GREEN, LIGHT_BLUE, RED, DARK_GREEN};
use carbide_core::widget::primitive::widget::WidgetExt;
use carbide_core::widget::primitive::spacer::Spacer;
use carbide_core::widget::primitive::edge_insets::EdgeInsets;
use self::calculator::calculator_button::CalculatorButton;
use calculator::calculator_state::CalculatorState;
use calculator::calculator_state::Operation;
use carbide_core::layout::CrossAxisAlignment;
use carbide_core::widget::types::spacer_direction::SpacerDirection;


#[macro_use]
extern crate carbide_derive;

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
                            Text::initialize(CommonState::GlobalState {
                            function: |global_state: &CalculatorState| {
                                global_state.get_upper_display()
                            },
                            function_mut: None,
                            latest_value: "0".to_string()
                        }).font_size(30.into()),
                            Text::initialize(CommonState::GlobalState {
                            function: |global_state: &CalculatorState| {
                                global_state.get_display()
                            },
                            function_mut: None,
                            latest_value: "0".to_string()
                        }).font_size(45.into())
                    ]).cross_axis_alignment(CrossAxisAlignment::End)
                ]).padding(EdgeInsets::all(10.0))
            ])
                .fill(DARK_GREEN)
                .frame(-1.0, 150.0),
            HStack::initialize(vec![

                CalculatorButton::new(
                    Text::initialize("".into())
                        .font_size(45.into())
                ).on_released(|b, s| println!("I am clicked")),

                CalculatorButton::new(
                    Text::initialize("".into())
                        .font_size(45.into())
                ),
                CalculatorButton::new(
                    Image::new(rust_image).resizeable().frame(45.0,45.0)
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