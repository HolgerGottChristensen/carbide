use carbide_core::draw::{Alignment, Color, Dimension};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let colors1 = vec![
        Color::Rgba(1.0, 0.0, 0.0, 1.0),
        Color::Rgba(0.0, 1.0, 0.0, 1.0),
        Color::Rgba(0.0, 0.0, 1.0, 1.0),
    ];

    application.set_scene(Window::new(
        "Icon gradient example",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Image::new_icon("images/rust.png")
                .color(Gradient::conic(colors1.clone(), Alignment::Center, Alignment::BottomTrailing)),
            Rectangle::new()
                .fill(Gradient::conic(colors1, Alignment::Center, Alignment::BottomTrailing))
                .frame(50.0, 50.0),
        )),
    ).close_application_on_window_close());

    application.launch();
}
