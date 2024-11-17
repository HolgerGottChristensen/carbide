use carbide_controls::ControlsExt;
use carbide_controls::picker::{Picker, RadioStyle};
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, StateValue};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

#[derive(Debug, Clone, PartialEq, StateValue)]
enum Flavor {
    Chocolate,
    Vanilla,
    Strawberry
}

fn main() {
    let state = LocalState::new(Flavor::Chocolate);

    let mut application = Application::new();

    application.set_scene(Window::new(
        "Picker example - Carbide",
        Dimension::new(400.0, 600.0),
        Picker::new("Flavor", state, (
            Text::new("Chocolate").tag(Flavor::Chocolate),
            Text::new("Vanilla").tag(Flavor::Vanilla),
            Text::new("Strawberry").tag(Flavor::Strawberry),
        ))
            .padding(EdgeInsets::all(40.0))
            .picker_style(RadioStyle)
    ));

    application.launch();
}
