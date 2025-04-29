use carbide_controls::button::{BorderedProminentStyle, Button};
use carbide_controls::ControlsExt;
use carbide_core::color::RED;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor::SecondarySystemBackground;
use carbide_core::state::LocalState;
use carbide_core::widget::*;
use carbide_dialogs::color_dialog::ColorDialog;
use carbide_dialogs::{DialogsExt, NativeStyle};
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let state = LocalState::new(RED);

    application.set_scene(
        Window::new(
            "Color Dialog example - Carbide",
            Dimension::new(400.0, 600.0),
            ZStack::new((
                Rectangle::new().fill(state.clone()),
                RoundedRectangle::new(CornerRadii::all(10.0))
                    .fill(SecondarySystemBackground)
                    .frame(200.0, 200.0),
                Button::new("Open dialog", move |ctx| {

                    ColorDialog::new(state.clone(), true)
                        .open(ctx.env);

                }).frame(120.0, 22.0)
                    .button_style(BorderedProminentStyle)
                    .color_dialog_style(NativeStyle)
            ))
        )
    );

    application.launch()
}
