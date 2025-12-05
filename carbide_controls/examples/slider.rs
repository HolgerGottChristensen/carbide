use carbide_controls::ControlsExt;
use carbide_controls::slider::{Slider, UnstyledStyle};
use carbide_controls::slider::SliderStepping::SmoothStepped;
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};
use carbide_core::state::ReadStateExtNew;

fn main() {
    let state1 = LocalState::new(60.0);
    let state2 = LocalState::new(5u32);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Slider example - Carbide",
        Dimension::new(400.0, 650.0),
        VStack::new((
            Text::new(state1.map(|a| format!("Current value: {:.2}", a)))
                .font_size(EnvironmentFontSize::Title2),
            Empty::new()
                .frame(20.0, 40.0),
            Group::new((
                Text::new("Slider 20.00 - 80.00"),
                Slider::new(state1.clone(), 20.0, 80.0)
                    .padding(20.0),
                Empty::new()
                    .frame(20.0, 20.0),
            )),
            Group::new((
                Text::new("Slider 0.00 - 100.00"),
                Slider::new(state1.clone(), 0.0, 100.0)
                    .padding(20.0)
                    .accent_color(EnvironmentColor::Green),
                Empty::new()
                    .frame(20.0, 20.0),
            )),
            Group::new((
                Text::new("Slider stepped 0.00 - 100.00"),
                Slider::new(state1.clone(), 0.0, 100.0)
                    .step(15.0)
                    .padding(20.0)
                    .accent_color(EnvironmentColor::Teal),
                Empty::new()
                    .frame(20.0, 20.0),
            )),
            Group::new((
                Text::new("Slider smooth stepped 0.00 - 100.00"),
                Slider::new(state1.clone(), 0.0, 100.0)
                    .step(SmoothStepped(15.0))
                    .padding(20.0)
                    .accent_color(EnvironmentColor::Purple),
                Empty::new()
                    .frame(20.0, 20.0),
            )),
            Group::new((
                Text::new(state2.map(|a| format!("Slider integer: {:.2}", a))),
                Slider::new(state2, 0, 25)
                    .accent_color(EnvironmentColor::Orange)
                    .padding(20.0),
                Empty::new().frame(20.0, 20.0),
            )),
            Group::new((
                Text::new("Slider disabled 20.00 - 80.00"),
                Slider::new(state1, 20.0, 80.0)
                    .enabled(false)
                    .padding(20.0),
            ))
        )).spacing(0.0)
            //.slider_style(UnstyledStyle)
    ));

    application.launch();
}
