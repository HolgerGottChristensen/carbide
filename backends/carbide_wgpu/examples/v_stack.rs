use carbide_core::draw::{Dimension};
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "VStack example",
            Dimension::new(600.0, 600.0),
            *VStack::new(vec![
                Rectangle::new()
                    .fill(EnvironmentColor::Accent)
                    .frame(100.0, 100.0)
                    .boxed(),
                Rectangle::new()
                    .fill(EnvironmentColor::Accent)
                    .frame(100.0, 100.0)
                    .boxed(),
                Rectangle::new()
                    .fill(EnvironmentColor::Accent)
                    .frame(100.0, 100.0)
                    .boxed(),
                Rectangle::new()
                    .fill(EnvironmentColor::Accent)
                    .frame(100.0, 100.0)
                    .boxed(),
            ]).spacing(10.0),
        ).close_application_on_window_close()
    );

    application.launch()
}
