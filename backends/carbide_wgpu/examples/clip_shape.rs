use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "ClipShape example - Carbide",
            Dimension::new(600.0, 600.0),
            Image::new("images/landscape.png")
                .clip_shape(Circle::new())
        )
    );

    application.launch()
}
