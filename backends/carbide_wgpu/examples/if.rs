use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_controls::ControlsExt;
use carbide_core::closure;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
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
            Button::new("Click to change the view above", closure!(|_| {
                *$condition = !*$condition;
            })).frame_fixed_height(45.0)
                .button_style(BorderedProminentStyle),
        ))
    ));

    application.launch();
}