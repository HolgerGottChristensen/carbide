use carbide_controls::TextInput;
use carbide_core::draw::Dimension;
use carbide_core::state::LocalState;
use carbide_core::widget::WidgetExt;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let title = LocalState::new("Window title example - Carbide".to_string());

    application.set_scene(Window::new(
        title.clone(),
        Dimension::new(400.0, 400.0),
        TextInput::new(title)
            .frame_fixed_width(200.0)
    ));

    application.launch();
}