use carbide_controls::{CheckBoxValue, PlainCheckBox};
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let checkbox_state1 = LocalState::new(CheckBoxValue::False);
    let checkbox_state2 = LocalState::new(false);
    let checkbox_state3 = LocalState::new(CheckBoxValue::Intermediate);
    let checkbox_state4 = LocalState::new(CheckBoxValue::True);

    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Plain Check Box Example - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![
            PlainCheckBox::new(checkbox_state1.clone()).border().boxed(),
            PlainCheckBox::new(checkbox_state2).border().boxed(),
            PlainCheckBox::new(checkbox_state3).border().boxed(),
            PlainCheckBox::new(checkbox_state4).border().boxed(),
        ])
            .spacing(10.0)
            .padding(EdgeInsets::all(40.0)),
    ).close_application_on_window_close());

    application.launch();
}
