extern crate carbide_wgpu;
extern crate futures;
extern crate env_logger;
extern crate carbide_core;

use carbide_core::widget::*;


use carbide_wgpu::window::Window;
use futures::executor::block_on;

use carbide_controls::{PlainButton};
use carbide_controls::List;
use carbide_core::color::{RED, GREEN};
use carbide_core::prelude::Uuid;


fn main() {
    env_logger::init();

    let icon_path = Window::<u32>::path_to_assets("images/rust_press.png");

    let mut window = block_on(Window::new("List Example - Carbide".to_string(), 800, 1200,Some(icon_path), 0));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    let list_model = (1..100).map(|i| format!("Number {}", i)).collect::<Vec<_>>();

    let list_model_state = CommonState::new_local_with_key(&list_model);

    let id_state = CommonState::new_local_with_key(&"Hello".to_string());

    window.set_widgets(
        List::new(
            Rectangle::initialize(vec![
                Text::initialize(Box::new(id_state.clone()))
            ]).fill(GREEN)
                .frame(SCALE.into(), 80.0.into()),
            Box::new(list_model_state))
            .id_state(Box::new(id_state.clone()))
            .frame(500.0.into(), SCALE.into())
            .frame(SCALE.into(), 900.0.into())


    );

    window.run_event_loop();

}