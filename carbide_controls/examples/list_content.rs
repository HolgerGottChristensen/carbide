use carbide_controls::{ControlsExt, List};
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, ReadState, State};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "List by content Example - Carbide",
        Dimension::new(400.0, 400.0),
        List::new_content((
            Text::new("Item 0"),
            Text::new("Item 1"),
            Text::new("Item 2"),
            Text::new("Item 3"),
            Text::new("Item 4"),
            Text::new("Item 5"),
            Text::new("Item 6"),
            Text::new("Item 7"),
            Text::new("Item 8"),
            Text::new("Item 9"),
        ))
            .border()
            .padding(50.0)
    ));

    application.launch();
}
