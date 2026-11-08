use carbide::draw::Dimension;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Image resizable example - Carbide",
        Dimension::new(400.0, 300.0),
        Image::new("images/lcabyg.png").resizeable()
    ));

    application.launch();
}
