use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Scroll example",
        Dimension::new(400.0, 600.0),
        Scroll::new(
            Image::new("images/landscape.png")
                .resizeable()
                .scaled_to_fill()
                .frame(500.0, 500.0)
                .boxed(),
        )
            .clip()
            .frame(250.0, 250.0),
    ).close_application_on_window_close());

    application.launch();
}
