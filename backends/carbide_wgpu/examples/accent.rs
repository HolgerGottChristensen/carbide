use carbide_core::color::{GREEN, RED};
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Accent example - Carbide",
            Dimension::new(600.0, 600.0),
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
        ).close_application_on_window_close()
    );

    application.launch()
}
