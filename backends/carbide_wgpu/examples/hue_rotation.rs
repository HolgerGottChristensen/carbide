use std::time::Duration;
use carbide_core::animation::{bounce_in_out, ease_in_out, linear};
use carbide_core::draw::{Color, Dimension};
use carbide_core::state::AnimatedState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let rotation = AnimatedState::custom(linear, None)
        .repeat()
        .duration(Duration::new(5, 0))
        .range(0.0, 1.0);

    application.set_scene(Window::new(
        "Hue rotation example",
        Dimension::new(800.0, 600.0),
        VStack::new((
            Image::new("images/landscape.png"),
            HStack::new((
                Rectangle::new()
                    .fill(Color::new_rgb(255, 0, 0))
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .fill(Color::new_rgb(0, 255, 0))
                    .frame(100.0, 100.0),
                Rectangle::new()
                    .fill(Color::new_rgb(0, 0, 255))
                    .frame(100.0, 100.0),
            ))
        )).hue_rotation(rotation)
    ).close_application_on_window_close());

    application.launch();
}
