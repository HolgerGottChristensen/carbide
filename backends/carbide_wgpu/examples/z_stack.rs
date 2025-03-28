use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "ZStack example - Carbide",
        Dimension::new(400.0, 600.0),
        ZStack::new((
            RoundedRectangle::new(10.0),
            Text::new("Hello world!")
        )).padding(40.0),
    ));

    application.launch();
}
