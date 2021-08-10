extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use futures::executor::block_on;

use carbide_controls::PlainButton;
use carbide_core::color::RED;
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::<u32>::path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Plain Button Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
        0,
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

    let hover_state = CommonState::new_local_with_key(&false);
    let pressed_state = CommonState::new_local_with_key(&false);
    let focus_state = CommonState::new_local_with_key(&Focus::Focused);

    window.set_widgets(
        VStack::new(vec![
            PlainButton::<bool, u32>::new(Rectangle::new(vec![]).fill(RED).frame(10.0, 10.0))
                .on_click(|_, _, f| {
                    *f += 1;
                })
                .hover(hover_state.clone())
                .pressed(pressed_state.clone())
                .focused(focus_state.clone())
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .frame(120.0, 70.0),
            Text::new(
                CommonState::GlobalState {
                    function: |state: &u32| state,
                    function_mut: |state| state,
                    latest_value: 0,
                }
                    .mapped(|val| val.to_string()),
            )
                .font_size(40),
            Text::new(hover_state.mapped(|m| format!("Is hovered: {}", m).to_string()))
                .font_size(40),
            Text::new(pressed_state.mapped(|m| format!("Is pressed: {}", m).to_string()))
                .font_size(40),
            Text::new(focus_state.mapped(|m| format!("Focus state: {:?}", m).to_string()))
                .font_size(40),
        ])
            .spacing(20.0),
    );

    window.run_event_loop();
}
