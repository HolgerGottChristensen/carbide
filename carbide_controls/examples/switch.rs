use carbide_controls::Switch;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let switch_state1 = LocalState::new(false);
    let switch_state2 = LocalState::new(true);
    let switch_state3 = LocalState::new(true);
    let switch_state4 = LocalState::new(false);
    let switch_state5 = LocalState::new(false);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Switch Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            Switch::new("Rectangle", switch_state1).boxed(),
            Switch::new("Circle", switch_state2)
                .accent_color(EnvironmentColor::Pink).boxed(),
            Switch::new("Triangle", switch_state3).boxed(),
            Switch::new("Star", switch_state4).boxed(),
            Empty::new().frame(10.0, 10.0).boxed(),
            Switch::new("Enabled", switch_state5.clone()).boxed(),
            Switch::new("Disabled 1", LocalState::new(true)).enabled(switch_state5.clone()).boxed(),
            Switch::new("Disabled 2", LocalState::new(false)).enabled(switch_state5).boxed(),
        ])
            .spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .accent_color(EnvironmentColor::Purple),
    ).close_application_on_window_close());

    application.launch();
}
