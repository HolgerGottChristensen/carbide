use carbide_core::prelude::EnvironmentColor;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Icon example".to_string(),
        800,
        800,
        Some(icon_path.clone()),
    );

    window.set_widgets(
        Rectangle::new()
    );

    window.launch();
}
