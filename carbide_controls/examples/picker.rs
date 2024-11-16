use carbide_controls::toggle::{ButtonStyle, CheckboxStyle, SwitchStyle, Toggle, ToggleValue};
use carbide_controls::ControlsExt;
use carbide_controls::picker::{Picker, RadioStyle, Tagged};
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let state = LocalState::new(0u32);

    let mut application = Application::new();

    application.set_scene(Window::new(
        "Picker example - Carbide",
        Dimension::new(400.0, 600.0),
        Picker::new("Test", state, vec![
            Text::new("Case 0").tag(0u32),
            //Text::new("Case 1").tag(1u32),
        ])
            .padding(EdgeInsets::all(40.0))
            .picker_style(RadioStyle)
    ));

    application.launch();
}
