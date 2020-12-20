mod calculator;

use conrod_wgpu::window::Window;
use futures::executor::block_on;
use conrod_core::window::TWindow;
use conrod_core::state::state::State;
use conrod_core::widget::primitive::v_stack::VStack;
use conrod_core::widget::{Text, Image, Rectangle, HStack, SCALE, Oval, Frame};
use conrod_core::widget::complex::SyncTest;
use conrod_core::color::{GREEN, LIGHT_BLUE, RED};
use conrod_core::widget::primitive::widget::WidgetExt;
use conrod_core::widget::primitive::spacer::{Spacer, SpacerDirection};
use conrod_core::widget::primitive::edge_insets::EdgeInsets;
use self::calculator::calculator_button::CalculatorButton;
use calculator::calculator_state::CalculatorState;
use calculator::calculator_state::Operation;

fn main() {
    env_logger::init();
    let mut window = block_on(Window::new("My first calculator".to_string(), 800, 800, CalculatorState::new()));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
    let rust_image = window.add_image("images/rust_press.png").unwrap();
    let rust_image1 = window.add_image("images/rust_hover.png").unwrap();
    let rust_image2 = window.add_image("images/rust.png").unwrap();


    window.set_widgets(
        VStack::initialize(vec![
            Rectangle::initialize(vec![

            ])
                .fill(GREEN)
                .frame(-1.0, 200.0),
            HStack::initialize(vec![
                CalculatorButton::new(rust_image)
                    .on_clicked(|b, s| println!("I am clicked")),
                CalculatorButton::new(rust_image1),
                CalculatorButton::new(rust_image),
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.set_operation(Operation::Div))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.append(7)),
                CalculatorButton::new(rust_image1).on_clicked(|b, s| s.append(8)),
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.append(9)),
                CalculatorButton::new(rust_image2).on_clicked(|b, s| s.set_operation(Operation::Mul))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.append(4)),
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.append(5)),
                CalculatorButton::new(rust_image1).on_clicked(|b, s| s.append(6)),
                CalculatorButton::new(rust_image2).on_clicked(|b, s| s.set_operation(Operation::Sub))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.append(1)),
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.append(2)),
                CalculatorButton::new(rust_image2).on_clicked(|b, s| s.append(3)),
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.set_operation(Operation::Add))
            ]).spacing(3.0),
            HStack::initialize(vec![
                CalculatorButton::new(rust_image1),
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.append(0)),
                CalculatorButton::new(rust_image),
                CalculatorButton::new(rust_image).on_clicked(|b, s| s.set_operation(Operation::None))
            ]).spacing(3.0),
        ]).spacing(3.0)
    );

    window.run_event_loop();

}