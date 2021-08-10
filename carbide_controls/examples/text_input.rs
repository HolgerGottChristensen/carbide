// #[macro_use]
// extern crate carbide_core;
// extern crate carbide_wgpu;
// extern crate env_logger;
// extern crate futures;
//
// use futures::executor::block_on;
// use serde::Deserialize;
// use serde::Serialize;
//
// use carbide_controls::{PopUpButton, TextInput};
// use carbide_core::color::RED;
// use carbide_core::state::environment_color::EnvironmentColor;
// use carbide_core::widget::*;
// use carbide_wgpu::window::Window;
//
// fn main() {
//     env_logger::init();
//
//     let icon_path = Window::<u32>::path_to_assets("images/rust_press.png");
//
//     let mut window = Window::new("Text Input Example - Carbide".to_string(), 800, 1200, Some(icon_path), 0);
//
//     window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();
//
//
//     let text_state = CommonState::new_local_with_key(&"Hello world!".to_string());
//
//
//     window.set_widgets(
//         TextInput::new(text_state)
//             .frame(235.0, 100.0)
//     );
//
//     window.run_event_loop();
// }
