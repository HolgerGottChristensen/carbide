use carbide_controls::toggle::{ButtonStyle, CheckboxStyle, SwitchStyle, Toggle, ToggleValue};
use carbide_controls::ControlsExt;
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let toggle_state1 = LocalState::new(ToggleValue::False);
    let toggle_state2 = LocalState::new(false);
    let toggle_state3 = LocalState::new(ToggleValue::Mixed);
    let toggle_state4 = LocalState::new(ToggleValue::True);

    let mut application = Application::new();

    application.set_scene(Window::new(
        "Toggle example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Toggle::new("Rectangle", toggle_state1.clone()),
            Toggle::new("Circle", toggle_state2.clone()),
            Toggle::new("Star", toggle_state3.clone()),
            Toggle::new("Triangle", toggle_state4.clone()),

            Empty::new().frame(1.0, 20.0),

            Toggle::new("Rectangle", toggle_state1).enabled(false),
            Toggle::new("Circle", toggle_state2).enabled(false),
            Toggle::new("Star", toggle_state3).enabled(false),
            Toggle::new("Triangle", toggle_state4).enabled(false),
            /*Proxy::new((
                Toggle::new("Rectangle", toggle_state1),
                Toggle::new("Circle", toggle_state2),
                Toggle::new("Star", toggle_state3),
                Toggle::new("Triangle", toggle_state4),
            )).enabled(false)*/
        ))
            .spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding(EdgeInsets::all(40.0))
            .toggle_style(SwitchStyle)
            .toggle_style(ButtonStyle)
            .toggle_style(CheckboxStyle)
    ));

    application.launch();
}
