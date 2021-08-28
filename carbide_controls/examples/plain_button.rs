extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::PlainButton;
use carbide_core::color::RED;
use carbide_core::focus::Focus;
use carbide_core::state::{LocalState, MapOwnedState, State};
use carbide_core::text::{FontFamily, FontStyle, FontWeight};
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Plain Button Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
    );

    let mut family = FontFamily::new("NotoSans");
    family.add_font(
        "fonts/NotoSans/NotoSans-Regular.ttf",
        FontWeight::Normal,
        FontStyle::Normal,
    );
    window.add_font_family(family);

    let hover_state = LocalState::new(false);
    let pressed_state = LocalState::new(false);
    let focus_state = LocalState::new(Focus::Focused);
    let counter_state = LocalState::new(0);
    let button_counter_state = counter_state.clone();

    window.set_widgets(
        VStack::new(vec![
            PlainButton::new(Rectangle::new(vec![]).fill(RED))
                .on_click(move |_: &mut _| {
                    *button_counter_state.clone().value_mut() += 1;
                })
                .hover(hover_state.clone())
                .pressed(pressed_state.clone())
                .focused(focus_state.clone())
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .frame(120.0, 70.0),
            Text::new(counter_state)
                .font_size(40),
            Text::new(hover_state)
                .font_size(40),
            Text::new(pressed_state)
                .font_size(40),
            Text::new(MapOwnedState::new(focus_state, |focus: &Focus| { format!("{:?}", focus) }))
                .font_size(40),
        ])
            .spacing(20.0),
    );

    window.launch();
}
