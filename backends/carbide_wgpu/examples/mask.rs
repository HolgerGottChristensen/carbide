use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Mask example",
            Dimension::new(600.0, 600.0),
            Image::new("images/landscape.png")
                .mask(VStack::new((
                    Text::new("Hello world!").font_size(42),
                    Image::new_icon("images/rust.png")
                )))
        ).close_application_on_window_close()
    );

    application.launch()
}
