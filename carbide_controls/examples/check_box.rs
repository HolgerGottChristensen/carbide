use carbide_controls::{CheckBox, CheckBoxValue, ControlsExt};
use carbide_core::accessibility::AccessibilityExt;
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let checkbox_state1 = LocalState::new(CheckBoxValue::False);
    let checkbox_state2 = LocalState::new(CheckBoxValue::False);
    let checkbox_state3 = LocalState::new(CheckBoxValue::Mixed);
    let checkbox_state4 = LocalState::new(true);

    let mut application = Application::new();//.with_asset_fonts();

    application.set_scene(Window::new(
        "Checkbox Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            CheckBox::new("Rectangle", checkbox_state1),
            CheckBox::new("Circle", checkbox_state2),
            CheckBox::new("Triangle", checkbox_state3),
            CheckBox::new("Star", checkbox_state4),
            Empty::new().frame(1.0, 20.0),
            CheckBox::new("Checked - Disabled", CheckBoxValue::True)
                .enabled(false),
            CheckBox::new("Indeterminate - Disabled", CheckBoxValue::Mixed)
                .enabled(false),
            CheckBox::new("Unchecked - Disabled", CheckBoxValue::False)
                .enabled(false),
        )).spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            //.enabled(false)
            .padding(EdgeInsets::all(40.0))
    ));

    application.launch();
}
