extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::Switch;
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::<String>::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Switch Example - Carbide".to_string(), 800, 1200, Some(icon_path), String::from("Hejsa"));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    let switch_state1 = CommonState::new_local_with_key(&false);
    let switch_state2 = CommonState::new_local_with_key(&true);
    let switch_state3 = CommonState::new_local_with_key(&false);

    window.set_widgets(
        SharedState::new(
            switch_state2.clone(),
            VStack::initialize(vec![
                Switch::new("Rectangle", switch_state1)
                    .frame(140.0, 26.0),
                Switch::new("Circle", switch_state2.clone())
                    .frame(140.0, 26.0),
                Switch::new("Triangle", switch_state2.clone())
                    .frame(140.0, 26.0),
                Switch::new("Star", switch_state3)
                    .frame(140.0, 26.0),
            ]).spacing(10.0).padding(EdgeInsets::all(40.0))
        )

    );

    window.run_event_loop();

}