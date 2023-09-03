use carbide_controls::{CheckBox, CheckBoxValue};
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let checkbox_state1 = LocalState::new(CheckBoxValue::False);
    let checkbox_state2 = LocalState::new(CheckBoxValue::False);
    let checkbox_state3 = LocalState::new(CheckBoxValue::Intermediate);
    let checkbox_state4 = LocalState::new(true);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Checkbox Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            CheckBox::new("Rectangle", checkbox_state1).boxed(),
            CheckBox::new("Circle", checkbox_state2).boxed(),
            CheckBox::new("Triangle", checkbox_state3).boxed(),
            CheckBox::new("Star", checkbox_state4).boxed(),
        ])
            .spacing(10.0)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .padding(EdgeInsets::all(40.0))
    ).close_application_on_window_close());

    application.launch();
}
