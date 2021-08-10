extern crate carbide_core;
extern crate carbide_wgpu;
extern crate env_logger;
extern crate futures;

use carbide_controls::List;
use carbide_core::color::GREEN;
use carbide_core::widget::*;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::<u32>::path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "List Example - Carbide".to_string(),
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

    let list_model = (1..100)
        .map(|i| format!("Number {}", i))
        .collect::<Vec<_>>();

    let list_model_state = CommonState::new_local_with_key(&list_model);

    let id_state = CommonState::new_local_with_key(&"Hello".to_string());

    window.set_widgets(
        List::new(
            Box::new(list_model_state),
            Rectangle::new(vec![Text::new(id_state.clone())])
                .fill(GREEN)
                .frame(SCALE, 80.0),
        )
            .id_state(Box::new(id_state.clone()))
            .frame(350.0, SCALE)
            .frame(SCALE, 500.0),
    );

    window.run_event_loop();
}
