use std::time::Duration;
use carbide_controls::{Button, capture, TextInput};
use carbide_core::animate;
use carbide_core::environment::Environment;
use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Match example".to_string(),
        400,
        600,
        Some(icon_path.clone()),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf"
    ]);
    window.add_font_family(family);

    let integer_state = LocalState::new(0);

    let middle = ZStack::new(vec![
        Rectangle::new().fill(EnvironmentColor::Yellow),
        TextInput::new(LocalState::new("Hello world!".to_string()))
            .padding(30.0)
    ]);

    window.set_widgets(
        VStack::new(vec![
            Button::new("Click to change the view below")
                .on_click(capture!([integer_state], |env: &mut Environment| {
                    *integer_state = (*integer_state + 1) % 3;
                })),
            Match::new(integer_state)
                .case(|a| matches!(a, 0), Rectangle::new().fill(EnvironmentColor::Blue))
                .case(|a| matches!(a, 1), middle)
                .case(|a| matches!(a, 2), Rectangle::new().fill(EnvironmentColor::Red))
        ]),

    );

    window.launch();
}
