use carbide_core::draw::Dimension;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    application.set_scene(Window::new(
        "Progress view example",
        Dimension::new(400.0, 600.0),
        VStack::new(vec![ProgressView::new(), ProgressView::new().size(50.0)]).spacing(10.0),
    ).close_application_on_window_close());

    application.launch();
}
