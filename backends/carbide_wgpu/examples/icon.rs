use carbide_core::prelude::EnvironmentColor;
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Icon example".to_string(),
        800,
        1200,
        Some(icon_path.clone()),
    );

    let image_id = window.add_image("images/rust.png");

    window.set_widgets(
        VStack::new(vec![
            Image::new_icon(image_id),
            Rectangle::new(vec![])
                .fill(EnvironmentColor::Accent)
                .frame(50, 50),
        ]).accent_color(EnvironmentColor::Red)
    );

    window.launch();
}
