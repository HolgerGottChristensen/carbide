use carbide::{Application, Scene3d, Window};
use carbide::draw::Dimension;
use carbide::widget::WidgetExt;

fn main() {
    carbide::init();

    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Cube example",
            Dimension::new(600.0, 600.0),
            Scene3d::new()
                .frame(300.0, 300.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
