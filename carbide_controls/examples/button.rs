extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::{Button, PlainButton};
use carbide_core::color::RED;
use carbide_core::focus::Focus;
use carbide_core::state::{BoolState, FocusState, I32State, LocalState, MapOwnedState, State};
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Button Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font_with_hints(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    let hover_state: BoolState = LocalState::new(false).into();
    let pressed_state: BoolState = LocalState::new(false).into();
    let focus_state: FocusState = LocalState::new(Focus::Focused).into();
    let counter_state: I32State = LocalState::new(0).into();
    let button_counter_state: I32State = counter_state.clone();

    window.set_widgets(
        VStack::new(vec![
            Button::new("Add 1")
                .on_click(move |_: &mut _| {
                    let mut temp = button_counter_state.clone();
                    *temp.value_mut() += 1;
                })
                .hover(hover_state.clone())
                .pressed(pressed_state.clone())
                .focused(focus_state.clone())
                .frame(60.0, 22.0),
            Text::new(counter_state.mapped(|count: &i32| { format!("Count: {}", count) }))
                .font_size(40),
            Text::new(hover_state.mapped(|hover: &bool| { format!("Hovered: {}", hover) }))
                .font_size(40),
            Text::new(pressed_state.mapped(|press: &bool| { format!("Pressed: {}", press) }))
                .font_size(40),
            Text::new(focus_state.mapped(|focus: &Focus| { format!("Focus: {:?}", focus) }))
                .font_size(40),
        ])
            .spacing(20.0),
    );

    window.launch();
}
