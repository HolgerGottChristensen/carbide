use std::time::Duration;

use carbide_core::draw::Dimension;
use carbide_core::state::{AnimatedState, ReadStateExtNew, Functor};
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    env_logger::init();

    let mut application = Application::new();

    let widget = Rectangle::new().frame(100.0, 100.0);

    let state =
        AnimatedState::linear(None)
            .repeat_alternate()
            .duration(Duration::new(1, 0))
            .range(0.0, 100.0)
            .map(|val: &f64| *val < 50.0);

    application.set_scene(
        Window::new(
            "Ignore example",
            Dimension::new(200.0, 300.0),
            VStack::new((
                Ignore::new(widget.clone()).render(state.clone()).border().boxed(),
                Ignore::new(widget.clone()).layout(state.clone()).border().boxed(),
            ))
        ).close_application_on_window_close()
    );

    application.launch()
}
