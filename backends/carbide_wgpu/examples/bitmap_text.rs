use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::text::PolarBearMarkup;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let text = Text::new_with_generator("# Rich text\nHello *world*, this is /italic/, _underlined_ and -striked-. We can even show 😀, and we support a list of fallback fonts!", PolarBearMarkup::new())
        .border()
        .border_width(1)
        .color(EnvironmentColor::Green)
        .padding(EdgeInsets::all(40.0));

    application.set_scene(Window::new(
        "Bitmap text example",
        Dimension::new(400.0, 600.0),
        text//.background(*Image::new("images/pixelgrid.png").scale_effect(0.5)).frame(400.0, 600.0),
    ).close_application_on_window_close());

    application.launch();
}
