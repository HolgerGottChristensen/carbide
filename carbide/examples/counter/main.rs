use carbide_controls::{Button, capture};
use carbide_wgpu::window::Window;
use carbide_core::prelude::*;
use carbide_core::text::FontFamily;
use carbide_core::window::TWindow;

fn main() {
    let mut window = Window::new(
        "My first counter",
        470 / 2,
        300,
        None,
    );

    let family = FontFamily::new_from_paths("NotoSans", vec![
        "fonts/NotoSans/NotoSans-Regular.ttf",
    ]);
    window.add_font_family(family);

    let counter = LocalState::new(0);

    let text = Text::new(counter.clone())
        .font_size(EnvironmentFontSize::LargeTitle);

    let button = Button::new("Increase counter")
        .on_click(capture!([counter], |_env: &mut Environment| {
                    *counter = *counter + 1;
                }))
        .frame(200, 30);

    window.set_widgets(
        VStack::new(vec![
            text,
            button
        ])
    );

    window.launch()
}