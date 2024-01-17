use chrono::Utc;
use carbide_core::draw::Dimension;
use carbide_core::impl_read_state;
use carbide_core::state::LocalState;
use carbide_core::widget::Text;
use carbide_fluent::LocalizedString;
use carbide_fluent::Localizable;
use carbide_wgpu::{Application, Window};

#[derive(Clone, Debug)]
enum Test {
    Var,
    Var2,
    Var3,
}

impl_read_state!(Test);

impl Localizable for Test {
    fn get(&self) -> &str {
        match self {
            Test::Var => "test-var",
            Test::Var2 => "test-var2",
            Test::Var3 => "test-var3",
        }
    }
}

fn main() {
    let mut application = Application::new();

    application.set_scene(Window::new(
        "LocalizedString - Carbide",
        Dimension::new(400.0, 600.0),
        Text::new(
            LocalizedString::new("response-msg")
                .arg("input", LocalState::new(Utc::now()))
                .arg("value", 22.2)
        )
    ).close_application_on_window_close());

    application.launch();
}