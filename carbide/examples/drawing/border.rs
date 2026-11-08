use carbide::draw::Dimension;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Border example - Carbide",
        Dimension::new(400.0, 600.0),
        Image::new("images/landscape.png")
            .clip()
            .frame(200.0, 200.0)
            .border()
            .border_width(10),
    ));

    application.launch();
}
