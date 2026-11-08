use carbide::draw::Dimension;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "Image example - Carbide",
        Dimension::new(800.0, 600.0),
        Image::new("images/landscape.png"),
    ));

    application.launch();
}
