use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Text with styling - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Text::new("Hello world!"),
            Text::new("Hello world!").bold(),
            Text::new("Hello world!").italic(),
        ))
    ));

    application.launch();
}
