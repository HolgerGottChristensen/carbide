use carbide_controls::{Button, capture, List, NavigationStack};
use carbide_core::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::{LocalState, State, StringState, TState, UsizeState};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::window::TWindow;
use carbide_wgpu::window::Window;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "NavigationStack Example - Carbide",
        600,
        600,
        Some(icon_path),
    );

    let family = FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    window.add_font_family(family);

    let mut stack = LocalState::new(vec![]);

    stack.clone().value_mut().push(
        ZStack::new(vec![
            Rectangle::new().fill(EnvironmentColor::Green),
            Button::new("PUSH").on_click(capture!([stack], | env: &mut Environment| {
                stack.push(Rectangle::new().fill(EnvironmentColor::Red) as Box<dyn Widget>)
            }))
        ]) as Box<dyn Widget>,
    );

    window.set_widgets(
        NavigationStack::new(stack),
    );

    window.launch();
}
