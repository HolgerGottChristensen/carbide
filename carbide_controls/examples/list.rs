use carbide_controls::List;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::{LocalState, State, StringState, TState, UsizeState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "List Example - Carbide",
        800,
        1200,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let list_model = (1..100)
        .map(|i| format!("Number {}", i))
        .collect::<Vec<_>>();

    let list_model_state = LocalState::new(list_model);

    fn delegate(item: StringState, _: UsizeState) -> Box<dyn Widget> {
        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::Green),
            Text::new(item),
        ]).frame_fixed_height(80.0)
    }

    window.set_widgets(
        List::new(list_model_state, delegate)
            .clip()
            .padding(50.0),
    );

    window.launch();
}
