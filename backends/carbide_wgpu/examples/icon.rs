use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Icon example",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            Image::new_icon("images/rust.png"),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(50.0, 50.0)
                .boxed(),
        ])
            .accent_color(EnvironmentColor::Green),
    ).close_application_on_window_close());

    application.launch();
}
