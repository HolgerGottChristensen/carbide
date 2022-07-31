use carbide_controls::{capture, Button};
use carbide_core::draw::Dimension;
use carbide_core::prelude::*;
use carbide_core::text::FontFamily;
use carbide_core::window::TWindow;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let family =
        FontFamily::new_from_paths("NotoSans", vec!["fonts/NotoSans/NotoSans-Regular.ttf"]);
    application.add_font_family(family);

    let counter = LocalState::new(0);

    let text = Text::new(counter.clone()).font_size(EnvironmentFontSize::LargeTitle);

    let button = Button::new("Increase counter")
        .on_click(capture!([counter], |_env: &mut Environment| {
            *counter = *counter + 1;
        }))
        .frame(200, 30);

    application.set_scene(Window::new(
        "My first counter",
        Dimension::new(235.0, 300.0),
        VStack::new(vec![text, button])
    ).close_application_on_window_close());

    application.launch()
}
