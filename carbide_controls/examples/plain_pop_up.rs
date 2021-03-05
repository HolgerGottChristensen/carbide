extern crate carbide_wgpu;
extern crate futures;
extern crate env_logger;
extern crate carbide_core;

use carbide_core::widget::*;


use carbide_wgpu::window::Window;
use futures::executor::block_on;

use carbide_controls::PlainPopUp;
use carbide_core::color::RED;
use carbide_core::state::environment_color::EnvironmentColor;


fn main() {
    env_logger::init();

    let icon_path = Window::<u32>::path_to_assets("images/rust_press.png");

    let mut window = block_on(Window::new("Plain Pop up Example - Carbide".to_string(), 800, 1200,Some(icon_path), 0));

    window.add_font("fonts/NotoSans/NotoSans-Regular.ttf").unwrap();

    let selected_index = CommonState::new_local_with_key(&0);

    window.set_widgets(
        OverlaidLayer::new ("overlay_test",
        VStack::initialize(vec![
            PlainPopUp::new(Box::new(selected_index))
                .padding(EdgeInsets::all(2.0))
                .border()
                .color(EnvironmentColor::Red.into())
                .clip()
                .frame(120.0.into(), 70.0.into()),
        ]).spacing(20.0)
    ));

    window.run_event_loop();

}