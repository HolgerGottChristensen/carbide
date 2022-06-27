use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::prelude::EnvironmentColor::{Green, Red, Yellow, Blue};
use carbide_core::widget::*;
use carbide_wgpu::window::*;

fn main() {
    env_logger::init();

    let icon_path = Window::relative_path_to_assets("images/rust_press.png");

    let mut window = Window::new(
        "Sub images example".to_string(),
        400,
        600,
        Some(icon_path.clone()),
    );

    let image_id = window.add_image_from_path("images/rust.png");

    window.set_widgets(
        VStack::new(vec![
            HStack::new(vec![
                Image::new_icon(image_id)
                    .source_rectangle(rect(0, 0))
                    .border()
                    .accent_color(Yellow),
                Image::new_icon(image_id)
                    .source_rectangle(rect(1, 0))
                    .border()
                    .accent_color(Red),
            ]),
            HStack::new(vec![
                Image::new_icon(image_id)
                    .source_rectangle(rect(0, 1))
                    .border()
                    .accent_color(Green),
                Image::new_icon(image_id)
                    .source_rectangle(rect(1, 1))
                    .border()
                    .accent_color(Blue),
            ])
        ])
    );

    window.launch();
}

fn rect(pos_x: u32, pos_y: u32) -> Rect {
    Rect::new(Position::new(72.0 * (pos_x as f64), 72.0 * (pos_y as f64)), Dimension::new(72.0, 72.0))
}
