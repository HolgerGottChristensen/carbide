use carbide_controls::{ControlsExt, Slider, UnstyledStyle};
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};
use carbide_core::state::ReadStateExtNew;

fn main() {
    let progress = LocalState::new(60.0);
    let progress2 = LocalState::new(5u32);
    let progress3 = LocalState::new(30.0);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Slider example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Text::new(progress.clone().map(|a| format!("Current value: {:.2}", a)))
                .font_size(EnvironmentFontSize::Title2),
            Empty::new()
                .frame(20.0, 40.0),
            Group::new((
                Text::new("Slider 20.00 - 80.00"),
                Slider::new(progress.clone(), 20.0, 80.0)
                    .padding(20.0),
                Empty::new()
                    .frame(20.0, 20.0),
            )),
            Group::new((
                Text::new("Slider 0.00 - 100.00"),
                Slider::new(progress.clone(), 0.0, 100.0)
                    .padding(20.0)
                    .accent_color(EnvironmentColor::Green),
                Empty::new()
                    .frame(20.0, 20.0),
            )),
            Group::new((
                Text::new("Slider stepped 0.00 - 100.0"),
                Slider::new(progress, 0.0, 100.0)
                    .step(Some(15.0))
                    .padding(20.0)
                    .accent_color(EnvironmentColor::Teal),
                Empty::new()
                    .frame(20.0, 20.0),
            )),
            Group::new((
                Text::new(
                    progress2
                        .clone()
                        .map(|a| format!("Slider integer: {:.2}", a)),
                ),
                Slider::new(progress2, 0, 25)
                    .accent_color(EnvironmentColor::Orange)
                    .padding(20.0),
                Empty::new().frame(20.0, 20.0),
            )),
            Group::new((
                Text::new(
                    progress3
                        .clone()
                        .map(|a| format!("Slider disabled: {:.2}", a)),
                ),
                Slider::new(progress3, 20.0, 100.0)
                    .enabled(false)
                    .padding(20.0),
            ))
        )).spacing(0.0)
            //.slider_style(UnstyledStyle)
    ));

    application.launch();
}
