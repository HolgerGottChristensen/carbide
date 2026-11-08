use carbide::controls::button::{BorderedProminentStyle, Button};
use carbide::controls::ControlsExt;
use carbide::color::RED;
use carbide::draw::Dimension;
use carbide::environment::EnvironmentColor::SecondarySystemBackground;
use carbide::state::LocalState;
use carbide::widget::*;
use carbide::dialogs::color_dialog::ColorDialog;
use carbide::dialogs::{DialogsExt, NativeStyle};
use carbide::{Application, Window};

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
