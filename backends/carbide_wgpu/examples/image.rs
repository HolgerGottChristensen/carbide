use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Image example - Carbide",
        Dimension::new(800.0, 600.0),
        Image::new("images/landscape.png"),
    ));

    application.launch();
}
