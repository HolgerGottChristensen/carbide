use std::f64::consts::PI;

use carbide_core::draw::Position;
use carbide_core::environment::*;
use carbide_core::text::*;
use carbide_core::widget::*;
use carbide_core::widget::canvas::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::path_to_assets("images/rust_press.png");

    let mut window = Window::new("Scroll example".to_string(), 800, 1200, Some(icon_path.clone()));

    window.set_widgets(
        IfElse::new(false)
            .when_true(Rectangle::new(vec![]).fill(EnvironmentColor::Orange))
            .when_false(Rectangle::new(vec![]).fill(EnvironmentColor::Green))
    );

    window.run_event_loop();
}