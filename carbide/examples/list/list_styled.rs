use std::collections::HashSet;
use carbide::controls::{ControlsExt, List};
use carbide::draw::{AutomaticStyle, Dimension};
use carbide::state::LocalState;
use carbide::widget::*;
use carbide::{Application, Window};

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
