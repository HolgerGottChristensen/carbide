use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use carbide_core::prelude::EnvironmentColor;
use carbide_core::state::{LocalState, Map1, State, StateExt};
use carbide_core::text::FontFamily;
use carbide_core::widget::*;
use carbide_core::{task, Color, SpawnTask};
use carbide_core::draw::Dimension;
use carbide_core::environment::Environment;
use carbide_macos::{ColorPanel, SavePanel};
use carbide_wgpu::{Application, Window};
use futures::future::Map;
use futures::FutureExt;
use oneshot::RecvError;
use carbide_core::asynchronous::StartStream;
use carbide_core::color::YELLOW;
use carbide_core::event::CustomEvent;

fn main() {

    let mut application = Application::new();

    fn window(child: Box<dyn Widget>) -> Box<Window> {
        Window::new(
            "Carbide MacOS color dialog example",
            Dimension::new(400.0, 600.0),
            child
        ).close_application_on_window_close()
    }

    let color = LocalState::new(YELLOW);
    let color_for_stream = color.clone();

    let widgets = MouseArea::new(Rectangle::new()
        .fill(color))
        .on_click(move |env: &mut Environment, _:_| {
            let color_for_stream = color_for_stream.clone();

            ColorPanel::new()
                .order_front(env)
                .start_stream(env, move |color: Color, env| {
                    let mut color_for_stream = color_for_stream.clone();
                    color_for_stream.set_value(color);
                    false
                });

        }).frame(100.0, 100.0);

    application.set_scene(
        window(widgets)
    );

    application.launch()
}
