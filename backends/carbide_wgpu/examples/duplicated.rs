use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new();

    let widget = Rectangle::new().frame(100.0, 100.0);

    let duplicated1 = Duplicated::new(widget);
    let duplicated2 = duplicated1.duplicate();

    application.set_scene(
        Window::new(
            "Duplicated example",
            Dimension::new(200.0, 300.0),
            VStack::new((
                duplicated1,
                duplicated2
            ))
        ).close_application_on_window_close()
    );

    application.launch()
}
