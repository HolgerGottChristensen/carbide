use carbide::closure;
use carbide::draw::Dimension;
use carbide::environment::*;
use carbide::state::LocalState;
use carbide::widget::*;
use carbide::{Application, Window};

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
        )
    );

    application.launch()
}
