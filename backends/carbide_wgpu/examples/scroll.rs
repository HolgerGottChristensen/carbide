use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Scroll example".to_string(),
        400,
        600,
        Some(icon_path.clone()),
    );

    let image_id = window.add_image_from_path("images/landscape.png");

    window.set_widgets(
        Scroll::new(
            Image::new(image_id)
                .resizeable()
                .scaled_to_fill()
                .frame(500.0, 500.0),
        )
        .clip()
        .frame(250.0, 250.0),
    );

    window.launch();
}
