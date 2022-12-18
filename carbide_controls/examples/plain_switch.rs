use carbide_controls::PlainSwitch;
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
        "Plain Switch Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            PlainSwitch::new("Rectangle", switch_state1).border(),
            PlainSwitch::new("Circle", switch_state2).border(),
            PlainSwitch::new("Triangle", switch_state3).border(),
            PlainSwitch::new("Star", switch_state4).border(),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ));

    application.launch();
}
