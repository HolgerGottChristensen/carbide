use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let elements = (
        Text::new("Test 1"),
        Text::new("Test 2"),
        Text::new("Test 3"),
        Text::new("Test 4"),
        Text::new("Test 5"),
    );

    application.set_scene(Window::new(
        "ForEach Widget example - Carbide",
        Dimension::new(400.0, 450.0),
        VStack::new(
            ForEach::widget(elements, |element: &dyn AnyWidget| {
                element.boxed().padding(10.0).border()
            })
        ),
    ));

    application.launch();
}
