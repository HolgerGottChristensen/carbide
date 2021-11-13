#[macro_use]
extern crate carbide_derive;

use carbide_core::widget::*;
use carbide_wgpu::window::*;

use crate::calculator::calculator_button::CalculatorButton;
use crate::calculator::calculator_state::{CalculatorState, Operation};

mod calculator;

fn main() {
    env_logger::init();
    let mut window = Window::new(
        "My first calculator".to_string(),
        400,
        550,
        None,
        CalculatorState::new(),
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    family.add_font(
        "fonts/NotoSans/NotoSans-Italic.ttf",
        FontWeight::Normal,
        FontStyle::Italic,
    );
    family.add_font(
        "fonts/NotoSans/NotoSans-Bold.ttf",
        FontWeight::Bold,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    let rust_image = window.add_image_from_path("images/rust_press.png").unwrap();

    window.set_widgets(
        VStack::new(vec![
            Rectangle::new_old(vec![HStack::new(vec![
                Spacer::new(),
                VStack::new(vec![
                    Text::new(CommonState::GlobalState {
                        function: |global_state: &CalculatorState| global_state.get_upper_display(),
                        function_mut: None,
                        latest_value: "0".to_string(),
                    })
                        .font_size(30),
                    Text::new(CommonState::GlobalState {
                        function: |global_state: &CalculatorState| global_state.get_display(),
                        function_mut: None,
                        latest_value: "0".to_string(),
                    })
                        .font_size(45),
                ])
                    .cross_axis_alignment(CrossAxisAlignment::End),
            ])
                .padding(EdgeInsets::all(10.0))])
                .fill(EnvironmentColor::Accent)
                .frame(SCALE, 150.0),
            HStack::new(vec![
                CalculatorButton::new(Text::new("").font_size(45))
                    .on_released(|_, _| println!("I am clicked")),
                CalculatorButton::new(Text::new("").font_size(45)),
                CalculatorButton::new(Image::new(rust_image).resizeable().frame(45.0, 45.0))
                    .on_released(|_, s| s.pop_char()),
                CalculatorButton::new(Text::new("/").font_size(45))
                    .on_released(|_, s| s.set_operation(Operation::Div)),
            ])
                .spacing(3.0),
            HStack::new(vec![
                CalculatorButton::new(Text::new("7").font_size(45)).on_released(|_, s| s.append(7)),
                CalculatorButton::new(Text::new("8").font_size(45)).on_released(|_, s| s.append(8)),
                CalculatorButton::new(Text::new("9").font_size(45)).on_released(|_, s| s.append(9)),
                CalculatorButton::new(Text::new("×").font_size(45))
                    .on_released(|_, s| s.set_operation(Operation::Mul)),
            ])
                .spacing(3.0),
            HStack::new(vec![
                CalculatorButton::new(Text::new("4").font_size(45)).on_released(|_, s| s.append(4)),
                CalculatorButton::new(Text::new("5").font_size(45)).on_released(|_, s| s.append(5)),
                CalculatorButton::new(Text::new("6").font_size(45)).on_released(|_, s| s.append(6)),
                CalculatorButton::new(Text::new("-").font_size(45))
                    .on_released(|_, s| s.set_operation(Operation::Sub)),
            ])
                .spacing(3.0),
            HStack::new(vec![
                CalculatorButton::new(Text::new("1").font_size(45)).on_released(|_, s| s.append(1)),
                CalculatorButton::new(Text::new("2").font_size(45)).on_released(|_, s| s.append(2)),
                CalculatorButton::new(Text::new("3").font_size(45)).on_released(|_, s| s.append(3)),
                CalculatorButton::new(Text::new("+").font_size(45))
                    .on_released(|_, s| s.set_operation(Operation::Add)),
            ])
                .spacing(3.0),
            HStack::new(vec![
                CalculatorButton::new(Text::new("").font_size(45)),
                CalculatorButton::new(Text::new("0").font_size(45)).on_released(|_, s| s.append(0)),
                CalculatorButton::new(Text::new("").font_size(45)),
                CalculatorButton::new(Text::new("=").font_size(45))
                    .on_released(|_, s| s.set_operation(Operation::Eq)),
            ])
                .spacing(3.0),
        ])
            .spacing(3.0),
    );

    window.launch();
}
