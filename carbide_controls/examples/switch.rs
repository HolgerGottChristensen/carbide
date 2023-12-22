use carbide_controls::{ControlsExt, Switch};
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let switch_state1 = LocalState::new(false);
    let switch_state2 = LocalState::new(true);
    let switch_state3 = LocalState::new(true);
    let switch_state4 = LocalState::new(false);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Switch Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            Switch::new("Rectangle", switch_state1),
            Switch::new("Circle", switch_state2),
            Switch::new("Triangle", switch_state3),
            Switch::new("Star", switch_state4),
            Empty::new().frame(10.0, 10.0),
            Switch::new("Checked - Disabled", true).enabled(false),
            Switch::new("Unchecked - Disabled", false).enabled(false),
        ))
            .spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::Start),
    ).close_application_on_window_close());

    application.launch();
}
