use carbide_core::draw::Dimension;
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let family =
        FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    application.add_font_family(family);

    application.set_scene(Window::new(
        "Background example",
        Dimension::new(400.0, 600.0),
        Text::new("Hello world!")
            .padding(5.0)
            .background(RoundedRectangle::new(3.0))
    ).close_application_on_window_close());

    application.launch();
}
