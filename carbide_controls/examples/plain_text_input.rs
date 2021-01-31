extern crate carbide_wgpu;
extern crate futures;
extern crate env_logger;
extern crate carbide_core;

use carbide_core::widget::*;


use carbide_wgpu::window::Window;
use futures::executor::block_on;

use carbide_controls::PlainTextInput;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = block_on(Window::new("Plain Text Input Example - Carbide".to_string(), 800, 1200,Some(icon_path), String::from("Hejsa")));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    let text_state = State::new_local_with_key(&"Hello World!".to_string());

    window.set_widgets(
        PlainTextInput::new(text_state)
            .padding(EdgeInsets::all(2.0))
            .border()
            .clip()
            .padding(EdgeInsets::all(50.0))
    );

    window.run_event_loop();

}