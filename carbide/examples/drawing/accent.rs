use carbide::color::{GREEN, RED};
use carbide::draw::Dimension;
use carbide::environment::*;
use carbide::widget::*;
use carbide::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Accent example - Carbide",
            Dimension::new(200.0, 200.0),
            VStack::new((
                Rectangle::new()
                    .frame(100.0, 30.0),
                Rectangle::new()
                    .frame(100.0, 30.0)
                    .accent_color(RED),
                Rectangle::new()
                    .frame(100.0, 30.0)
                    .accent_color(EnvironmentColor::Red)
            )).accent_color(GREEN)
        )
    );

    application.launch()
}
