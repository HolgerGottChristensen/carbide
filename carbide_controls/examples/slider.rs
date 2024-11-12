use carbide_controls::{ControlsExt, Slider};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};
use carbide_core::state::ReadStateExtNew;

fn main() {
    let progress = LocalState::new(80.0);
    let progress2 = LocalState::new(5u32);
    let progress3 = LocalState::new(30.0);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Slider example - Carbide",
        Dimension::new(400.0, 400.0),
        VStack::new((
            Text::new(
                progress
                    .clone()
                    .map(|a| format!("Slider value: {:.2}", a)),
            ),
            Slider::new(progress, 20.0, 100.0)
                .padding(20.0),
            Empty::new()
                .frame(20.0, 20.0),
            Text::new(
                progress2
                    .clone()
                    .map(|a| format!("Slider step value: {:.2}", a)),
            ),
            Slider::new(progress2, 0, 25)
                .accent_color(EnvironmentColor::Orange)
                .padding(20.0),
            Empty::new().frame(20.0, 20.0),
            Text::new(
                progress3
                    .clone()
                    .map(|a| format!("Slider disabled value: {:.2}", a)),
            ),
            Slider::new(progress3, 20.0, 100.0)
                .enabled(false)
                .padding(20.0),
        ))
    ));

    application.launch();
}
