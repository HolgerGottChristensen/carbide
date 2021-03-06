extern crate carbide_wgpu;
extern crate futures;
extern crate env_logger;
extern crate carbide_core;

use carbide_core::widget::*;


use carbide_wgpu::window::Window;
use futures::executor::block_on;

use carbide_controls::PlainButton;
use carbide_core::color::RED;


fn main() {
    env_logger::init();

    let icon_path = Window::<u32>::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Plain Button Example - Carbide".to_string(), 800, 1200,Some(icon_path), 0);

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    let hover_state = CommonState::new_local_with_key(&false);
    let pressed_state = CommonState::new_local_with_key(&false);
    let focus_state = CommonState::new_local_with_key(&Focus::Focused);

    window.set_widgets(
        VStack::initialize(vec![
            PlainButton::<bool, u32>::new(Rectangle::initialize(vec![])
                .fill(RED)
                .frame(10.0,10.0)
            )
                .on_click(|_,_, f| {
                    *f += 1;
                }).hover(hover_state.clone().into())
                .pressed(pressed_state.clone().into())
                .focused(focus_state.clone().into())
                .padding(EdgeInsets::all(2.0))
                .border()
                .clip()
                .frame(120.0, 70.0),
            Text::new(CommonState::GlobalState {
                function: |state: &u32| { state.to_string()},
                function_mut: None,
                latest_value: "0".to_string()
            }).font_size(40),
            Text::new(hover_state.mapped(|m|{
                format!("Is hovered: {}", m).to_string()
            })).font_size(40),
            Text::new(pressed_state.mapped(|m|{
                format!("Is pressed: {}", m).to_string()
            })).font_size(40),
            Text::new(focus_state.mapped(|m|{
                format!("Focus state: {:?}", m).to_string()
            })).font_size(40)
        ]).spacing(20.0)

    );

    window.run_event_loop();

}