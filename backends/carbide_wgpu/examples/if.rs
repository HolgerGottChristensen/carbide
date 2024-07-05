use carbide_controls::{Button, TextInput};
use carbide_core as carbide; // Required only in internal examples
use carbide_core::a;
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_macro::ui;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new()
        .with_asset_fonts();

    let condition = LocalState::new(false);

    let widget = ui!(
        if condition {
            Rectangle::new().fill(EnvironmentColor::Blue)
        } else {
            Rectangle::new().fill(EnvironmentColor::Red)
        }
    );

    application.set_scene(Window::new(
        "If - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            widget,
            Button::new_primary("Click to change the view above", a!(|_, _| {
                *$condition = !*$condition;
            })).frame_fixed_height(45.0),
        ))
    ).close_application_on_window_close());


    application.launch();
}