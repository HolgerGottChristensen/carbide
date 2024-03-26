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
            /*Image::new("images/landscape.png")
                .mask(Rectangle::new().frame(300.0, 300.0)
                    .mask(
                        Circle::new().frame(320.0, 320.0)
                    ))*/
        ).close_application_on_window_close()
    );

    application.launch()
}
