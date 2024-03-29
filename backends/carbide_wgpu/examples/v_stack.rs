use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "VStack - Carbide",
            Dimension::new(600.0, 600.0),
            HStack::new((
                VStack::new((
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                )).spacing(10.0)
                    .border(),
                VStack::new((
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Spacer::new(),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                )).spacing(10.0)
                    .border(),
                VStack::new((
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Spacer::new(),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Spacer::new(),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                )).spacing(10.0)
                    .border(),
                VStack::new((
                    Spacer::new(),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Spacer::new(),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Spacer::new(),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Spacer::new(),
                )).spacing(10.0)
                    .border(),
                VStack::new((
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Spacer::new(),
                    Spacer::new(),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                    Spacer::new(),
                    Rectangle::new()
                        .fill(EnvironmentColor::Accent)
                        .frame(50.0, 50.0),
                )).spacing(10.0)
                    .border(),
            )).spacing(20.0).padding(50.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
