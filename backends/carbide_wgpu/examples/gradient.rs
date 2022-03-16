use carbide_core::Color;
use carbide_core::color::{BLUE, GREEN, RED, YELLOW};
use carbide_core::draw::alignment::Alignment;
use carbide_core::draw::Position;
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
            .fill(Gradient::conic_ratios(vec![
                (Color::Rgba(1.0, 0.0, 0.0, 1.0), 0.0),
                //(Color::Rgba(1.0, 0.0, 0.0, 1.0), 0.5),
                //(Color::Rgba(1.0, 0.5, 0.0, 1.0), 0.5),
                (Color::Rgba(1.0, 0.5, 0.0, 1.0), 1.0),
            ], Position::new(0.0, 0.0), Position::new(40.0, 20.0))
                .repeat()
            )
            .frame(100.0, 100.0)
    );

    window.launch();
}
