use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let elements = ForEach::new(0..1_000_000_000, |_, idx| {
        ZStack::new((
            Rectangle::new(),
            Text::new(idx)
        ))
    });

    application.set_scene(Window::new(
        "ForEach Widget Lazy example - Carbide",
        Dimension::new(400.0, 450.0),
        Scroll::new(
            LazyVStack::new(
                ForEach::widget(elements, |a: &dyn AnyWidget| {
                    a.boxed().border().frame_fixed_height(30.0)
                })
            ).spacing(3.0)
        )
            .clip()
            .border()
            .padding(50.0)
    ));

    application.launch();
}
