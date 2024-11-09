use carbide_controls::{CheckBoxValue, CheckboxStyle, ControlsExt, PlainCheckBox, SwitchStyle};
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let checkbox_state1 = LocalState::new(CheckBoxValue::False);
    let checkbox_state2 = LocalState::new(false);
    let checkbox_state3 = LocalState::new(CheckBoxValue::Mixed);
    let checkbox_state4 = LocalState::new(CheckBoxValue::True);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Plain Check Box Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            PlainCheckBox::new(checkbox_state1).border(),
            PlainCheckBox::new(checkbox_state2).border()
                .toggle_style(CheckboxStyle),
            PlainCheckBox::new(checkbox_state3).border(),
            PlainCheckBox::new(checkbox_state4).border(),
        ))
            .spacing(10.0)
            .toggle_style(SwitchStyle)
            .padding(EdgeInsets::all(40.0)),
    ).close_application_on_window_close());

    application.launch();
}
