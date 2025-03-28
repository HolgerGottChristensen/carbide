use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Clip example - Carbide",
        Dimension::new(400.0, 600.0),
        Image::new("images/landscape.png")
            .scaled_to_fill()
            .frame(800.0, 800.0)
            .clip()
            .frame(200.0, 200.0),
    ));

    application.launch();
}
