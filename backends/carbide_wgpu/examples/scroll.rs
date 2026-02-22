use carbide_core::draw::{AutomaticStyle, DebugStyle, Dimension};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "Scroll example - Carbide",
        Dimension::new(400.0, 600.0),
        Scroll::new(
            Image::new("images/landscape.png")
                .resizeable()
                .scaled_to_fill()
                .clip()
                .frame(500.0, 500.0)
                .boxed(),
        )
            .clip()
            .frame(250.0, 250.0)
            .border()
            //.vertical_scroll_style(DebugStyle)
            //.horizontal_scroll_style(DebugStyle)
    ));

    application.launch();
}
