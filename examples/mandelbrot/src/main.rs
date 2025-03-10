use carbide::draw::Dimension;
use carbide::state::State;
use carbide::widget::WidgetExt;
use carbide::{Application, Window};

use crate::mandelbrot::Mandelbrot;

mod mandelbrot;

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(
        Window::new(
            "Mandelbrot",
            Dimension::new(800.0, 800.0),
            Mandelbrot::new()
                //.border()
                //.padding(200.0)
        )
    );

    application.launch()
}