use carbide_controls::PlainSlider;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, ReadStateExtNew};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let progress = LocalState::new(80.0); // Test bounds of slider.

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Plain slider example",
        Dimension::new(400.0, 500.0),
        VStack::new((
            Text::new(
                progress
                    .clone()
                    .map(|a| format!("Small slider value: {:.2}", a)),
            ),
            PlainSlider::new(progress.clone(), 40.0, 80.0)
                .border()
                .color(EnvironmentColor::Yellow)
                .padding(20.0),
            Empty::new().frame(20.0, 20.0),
            Text::new(
                progress
                    .clone()
                    .map(|a| format!("Slider value: {:.2}", a)),
            ),
            PlainSlider::new(progress.clone(), 20.0, 100.0)
                .border()
                .color(EnvironmentColor::Yellow)
                .padding(20.0),
            Empty::new().frame(20.0, 20.0),
            Text::new(
                progress
                    .clone()
                    .map(|a| format!("Slider step value: {:.2}", a)),
            ),
            PlainSlider::new(progress, 20.0, 100.0)
                .step(Some(15.0))
                .border()
                .color(EnvironmentColor::Yellow)
                .padding(20.0),
        ))
    ).close_application_on_window_close());

    application.launch();
}
