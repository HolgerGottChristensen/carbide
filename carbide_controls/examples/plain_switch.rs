use carbide_controls::PlainSwitch;
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    // It is important to use LocalState or some other state because
    // otherwise it will not be shared between the different components
    // of the plain switch.
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
            PlainSwitch::new(switch_state1).border().boxed(),
            PlainSwitch::new(switch_state2).border().boxed(),
            PlainSwitch::new(switch_state3).border().boxed(),
            PlainSwitch::new(switch_state4).border().boxed(),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ).close_application_on_window_close());

    application.launch();
}
