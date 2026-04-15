use std::collections::HashSet;
use carbide_controls::{ControlsExt, List};
use carbide_core::draw::{AutomaticStyle, Dimension};
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let selection = LocalState::new(HashSet::new());

    application.set_scene(Window::new(
        "List Styled Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new_selectable(0..10_000, selection, |item, _| {
            Text::new(item)
        })
            .list_style(AutomaticStyle)
    ));

    application.launch();
}
