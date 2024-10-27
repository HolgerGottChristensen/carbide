use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::environment::*;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let state = LocalState::new("Hejsa!".to_string());

    application.set_scene(
        Window::new(
            "MouseArea - Carbide",
            Dimension::new(600.0, 600.0),
            Rectangle::new()
                .fill(EnvironmentColor::Accent)
                .frame(100.0, 100.0)
                .on_click(closure!(|_| {
                    println!("{}", $state);
                }))
        ).close_application_on_window_close()
    );

    application.launch()
}
