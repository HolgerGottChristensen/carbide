use carbide_controls::picker::{InlineStyle, MenuStyle, Picker, SegmentedStyle};
use carbide_controls::ControlsExt;
use carbide_core::draw::Dimension;
use carbide_core::state::{LocalState, StateValue};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, StateValue)]
enum Flavor {
    Chocolate,
    Vanilla,
    Strawberry
}

fn main() {
    let state = LocalState::new(Flavor::Chocolate);
    let state2 = LocalState::new(Some(Flavor::Chocolate));
    let state3 = LocalState::new(HashSet::<Flavor>::new());

    let mut application = Application::new();

    application.set_scene(Window::new(
        "Picker example - Carbide",
        Dimension::new(400.0, 600.0),
        Picker::new("Flavor", state, (
            Text::new("Chocolate").tag(Flavor::Chocolate),
            Text::new("Vanilla").tag(Flavor::Vanilla),
            Text::new("Strawberry").tag(Flavor::Strawberry),
        ))
            .padding(40.0)
            .picker_style(MenuStyle)
            .picker_style(InlineStyle)
            .picker_style(SegmentedStyle)
            //.accent_color(EnvironmentColor::Orange)
            //.enabled(false)
    ));

    application.launch();
}
