use carbide_core::draw::theme::Theme;
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "Theme example - Carbide",
            Dimension::new(600.0, 600.0),
            ZStack::new((
                Rectangle::new(),
                VStack::new((
                    ZStack::new((
                        Rectangle::new()
                            .fill(EnvironmentColor::SystemFill),
                        Text::new("System Theme")
                            .font_size(42)
                    )).frame(400.0, 150.0),
                    ZStack::new((
                        Rectangle::new()
                            .fill(EnvironmentColor::SystemFill),
                        Text::new("Dark Theme")
                            .font_size(42)
                    )).frame(400.0, 150.0)
                        .theme(Theme::Dark),
                    ZStack::new((
                        Rectangle::new()
                            .fill(EnvironmentColor::SystemFill),
                        Text::new("Light Theme")
                            .font_size(42)
                    )).frame(400.0, 150.0)
                        .theme(Theme::Light),
                ))
            ))
        )
    );

    application.launch()
}
