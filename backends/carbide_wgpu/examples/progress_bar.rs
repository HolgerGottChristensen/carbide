use std::time::Duration;
use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{AnimatedState, F64State, LocalState};
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Progress bar example".to_string(),
        800,
        800,
        Some(icon_path.clone()),
    );

    let progress = AnimatedState::linear(None).repeat().duration(Duration::from_secs(5)).range(0.0, 1.0);

    window.set_widgets(
        ProgressBar::new(progress).padding(20.0).accent_color(EnvironmentColor::Yellow)
    );

    window.launch();
}
