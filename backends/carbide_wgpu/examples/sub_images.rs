use carbide_core::draw::{Dimension, Position, Rect};
use carbide_core::environment::EnvironmentColor::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Sub images example",
        Dimension::new(400.0, 600.0),
        VStack::new((
            HStack::new((
                Image::new_icon("images/rust.png")
                    .source_rectangle(rect(0, 0))
                    .border()
                    .foreground_color(Yellow),
                Image::new_icon("images/rust.png")
                    .source_rectangle(rect(1, 0))
                    .border()
                    .foreground_color(Red),
            )),
            HStack::new((
                Image::new_icon("images/rust.png")
                    .source_rectangle(rect(0, 1))
                    .border()
                    .foreground_color(Green),
                Image::new_icon("images/rust.png")
                    .source_rectangle(rect(1, 1))
                    .border()
                    .foreground_color(Blue),
            )),
        ))
    ).close_application_on_window_close());

    application.launch();
}

fn rect(pos_x: u32, pos_y: u32) -> Rect {
    Rect::new(
        Position::new(72.0 * (pos_x as f64), 72.0 * (pos_y as f64)),
        Dimension::new(72.0, 72.0),
    )
}
