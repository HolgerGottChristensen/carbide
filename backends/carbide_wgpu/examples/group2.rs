use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    fn delegate(sequence: impl Sequence) -> impl Sequence {
        (
            Rectangle::new().fill(EnvironmentColor::Green).frame(100.0, 50.0),
            Text::new(sequence.len()).font_size(EnvironmentFontSize::LargeTitle),
            ForEach::identity(sequence)
        )
    }

    application.set_scene(Window::new(
        "Group 2 example - Carbide",
        Dimension::new(300.0, 600.0),
        VStack::new(
            Group::sequence((
                Rectangle::new().frame(100.0, 50.0),
                Rectangle::new().frame(100.0, 50.0),
                Rectangle::new().frame(100.0, 50.0),
                Rectangle::new().frame(100.0, 50.0),
                Rectangle::new().frame(100.0, 50.0),
                Rectangle::new().frame(100.0, 50.0),
            ), delegate)
        )
    ));

    application.launch();
}
