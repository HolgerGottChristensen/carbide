use carbide_core::prelude::EnvironmentColor;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Progress view example".to_string(),
        400,
        600,
        Some(icon_path.clone()),
    );

    window.set_widgets(
        VStack::new(vec![ProgressView::new(), ProgressView::new().size(50.0)]).spacing(10.0),
    );

    window.launch();
}
