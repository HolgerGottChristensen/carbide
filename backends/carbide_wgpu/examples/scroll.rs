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

    let image_id = window.add_image("images/rust_press.png");

    window.set_widgets(
        Scroll::new(
            Image::new(image_id)
                .resizeable()
                .frame(500.0, 500.0)
        ).clip().frame(250.0, 250.0)
            .border()
    );

    window.run_event_loop();
}