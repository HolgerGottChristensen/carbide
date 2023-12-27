use carbide_core::asynchronous::StartStream;
use carbide_core::color::{Color, YELLOW};
use carbide_core::draw::Dimension;
use carbide_core::environment::Environment;
use carbide_core::state::{LocalState, State};
use carbide_core::widget::*;
use carbide_macos::ColorPanel;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let color = LocalState::new(YELLOW);
    let color_for_stream = color.clone();

    application.set_scene(
        Window::new(
            "MacOS Color Dialog - Carbide",
            Dimension::new(400.0, 600.0),
            MouseArea::new(Rectangle::new().fill(color))
                .on_click(move |env: &mut Environment, _:_| {
                    let color_for_stream = color_for_stream.clone();

                    ColorPanel::new()
                        .set_shows_alpha(true)
                        .order_front(env)
                        .start_stream(move |color: Color, env| {
                            let mut color_for_stream = color_for_stream.clone();
                            color_for_stream.set_value(color);
                            false
                        });

                }).frame(100.0, 100.0)
        ).close_application_on_window_close()
    );

    application.launch()
}
