use carbide_controls::{ControlsExt, List};
use carbide_controls::list::PlainStyle;
use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "List Example - Carbide",
        Dimension::new(400.0, 600.0),
        List::new(0..10_000, |item, _| {
            ZStack::new((
                Rectangle::new(),
                Text::new(item),
            )).frame_fixed_height(20.0)
        }).padding(50.0)
            .list_style(PlainStyle(1.0))
    ));

    application.launch();
}
