use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    application.set_scene(
        Window::new(
            "VGrid - Carbide",
            Dimension::new(600.0, 600.0),
            VGrid::new((
                Rectangle::new(),
                Rectangle::new(),
                Rectangle::new(),
                Rectangle::new(),
            ), vec![
                VGridColumn::Adaptive(100.0)
            ]).border().padding(10.0),
        ).close_application_on_window_close()
    );

    application.launch()
}
