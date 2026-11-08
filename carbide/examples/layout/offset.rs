use carbide::draw::Dimension;
use carbide::environment::*;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Offset example - Carbide",
            Dimension::new(200.0, 200.0),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 100.0)
                .offset(25.0, 25.0)
                .border()
        )
    );

    application.launch()
}
