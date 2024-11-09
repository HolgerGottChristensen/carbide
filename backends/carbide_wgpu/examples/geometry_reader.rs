use carbide_core::draw::{Dimension, Rect};
use carbide_core::state::{LocalState, Map1};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let geometry = LocalState::new(Rect::default());

    application.set_scene(
        Window::new(
            "GeometryReader example",
            Dimension::new(600.0, 600.0),
            GeometryReader::new(
                geometry.clone(),
                ZStack::new((
                    Rectangle::new(),
                    Text::new(Map1::read_map(geometry, |g| {
                        format!("X: {}, Y: {}, W: {}, H: {}", g.position.x, g.position.y, g.dimension.width, g.dimension.height)
                    }))
                )),
            ).padding(30.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
