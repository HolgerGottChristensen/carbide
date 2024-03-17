use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Image resize example",
        Dimension::new(400.0, 300.0),
        Image::new("images/lcabyg.png").resizeable()
    ).close_application_on_window_close());

    application.launch();
}
