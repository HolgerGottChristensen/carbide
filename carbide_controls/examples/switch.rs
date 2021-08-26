extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::Switch;
use carbide_core::state::global_state::GlobalState;
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::<bool>::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Switch Example - Carbide".to_string(),
        800,
        1200,
        Some(icon_path),
        false,
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

    let switch_state1 = CommonState::new_local_with_key(&false);
    let switch_state2 = GlobalState::<bool, bool>::new(
        |global_state: &bool| -> &bool { global_state },
        |global_state: &mut bool| -> &mut bool { global_state },
    );
    let switch_state3 = CommonState::new_local_with_key(&false);

    window.set_widgets(SharedState::new(
        switch_state2.clone(),
        VStack::new(vec![
            Switch::new("Rectangle", switch_state1).frame(140.0, 26.0),
            Switch::new("Circle", switch_state2.clone()).frame(140.0, 26.0),
            Switch::new("Triangle", switch_state2.clone()).frame(140.0, 26.0),
            Switch::new("Star", switch_state3).frame(140.0, 26.0),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ));

    window.launch();
}
