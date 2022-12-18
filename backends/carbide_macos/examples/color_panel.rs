use carbide_core::Color;
use carbide_core::asynchronous::StartStream;
use carbide_core::color::YELLOW;
use carbide_core::draw::Dimension;
use carbide_core::environment::Environment;
use carbide_core::state::{LocalState, State};
use carbide_core::widget::*;
use carbide_macos::ColorPanel;
use carbide_wgpu::{Application, Window};

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
                .set_shows_alpha(true)
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
