use carbide_controls::Slider;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, StateExt};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let progress = LocalState::new(80.0);
    let progress2 = LocalState::new(80.0);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Slider example",
        Dimension::new(400.0, 400.0),
        VStack::new(vec![
            Text::new(
                progress
                    .clone()
                    .map(|a: &f64| format!("Slider value: {:.2}", a)),
            ),
            Slider::new(progress, 20.0, 100.0).padding(20.0),
            Empty::new().frame(20.0, 20.0),
            Text::new(
                progress2
                    .clone()
                    .map(|a: &f64| format!("Slider step value: {:.2}", a)),
            ),
            Slider::new(progress2, 20.0, 100.0)
                .step(5.0)
                .accent_color(EnvironmentColor::Orange)
                .padding(20.0),
        ])
    ).close_application_on_window_close());

    application.launch();
}
