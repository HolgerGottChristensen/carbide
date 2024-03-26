use std::time::Duration;

use carbide_core::animation::{bounce_in, bounce_out, elastic_out, elastic_out_hard, linear};
use carbide_core::draw::{Alignment, Color, ColorSpace, Dimension};
use carbide_core::state::AnimatedState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let rotation = AnimatedState::custom(bounce_out, None)
        .repeat()
        .duration(Duration::new(5, 0))
        .range(0.0, 1.0);

    application.set_scene(Window::new(
        "Hue rotation example - Carbide",
        Dimension::new(700.0, 600.0),
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
            )),
            Rectangle::new()
                .fill(Gradient::linear(vec![
                    Color::new_rgb(255, 0, 255),
                    Color::new_rgb(255, 0, 0),
                    Color::new_rgb(255, 255, 0),
                    Color::new_rgb(0, 255, 0),
                    Color::new_rgb(0, 255, 255),
                    Color::new_rgb(0, 0, 255),
                    Color::new_rgb(255, 0, 255),
                ], Alignment::Leading, Alignment::Trailing).color_space(ColorSpace::HSL))
                .frame(320.0, 30.0),
        )).hue_rotation(rotation)
    ).close_application_on_window_close());

    application.launch();
}
