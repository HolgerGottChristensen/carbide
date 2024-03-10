use carbide_core::draw::{Alignment, Color, Dimension};
use carbide_core::environment::EnvironmentColor;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let colors1 = vec![
        Color::Rgba(1.0, 0.0, 0.0, 1.0),
        Color::Rgba(0.0, 1.0, 0.0, 1.0),
        Color::Rgba(0.0, 0.0, 1.0, 1.0),
    ];

    application.set_scene(Window::new(
        "Icon example",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Image::new_icon("images/rust.png")
                .color(Gradient::conic(colors1, Alignment::Center, Alignment::BottomTrailing))
                .foreground_color(EnvironmentColor::Accent),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(50.0, 50.0),
        )).accent_color(EnvironmentColor::Green),
    ).close_application_on_window_close());

    application.launch();
}
