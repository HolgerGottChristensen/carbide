use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    fn delegate(child: impl Widget) -> impl Widget {
        child.padding(10.0).border()
    }

    application.set_scene(Window::new(
        "ForEach Widget example - Carbide",
        Dimension::new(600.0, 450.0),
        VStack::new(ForEach::widget(
            (
                Text::new("Test 1"),
                Text::new("Test 2"),
                Text::new("Test 3"),
            ),
            delegate,
        ))
        .spacing(10.0),
    ));

    application.launch();
}
