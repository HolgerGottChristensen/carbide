use chrono::Utc;
use carbide_controls::{ControlsExt, PopUpButton, Slider, TextInput};
use carbide_core::draw::Dimension;
use carbide_core::environment::{EnvironmentColor, EnvironmentFontSize};
use carbide_core::impl_read_state;
use carbide_core::state::LocalState;
use carbide_core::widget::{Text, VStack, WidgetExt};
use carbide_fluent::{Arg, LocalizedArg, LocalizedString};
use carbide_fluent::Localizable;
use carbide_wgpu::{Application, Window};

#[derive(Clone, Debug, PartialEq)]
enum Gender {
    Male,
    Female,
    Other,
}

impl Arg for Gender {
    fn into(&self) -> LocalizedArg {
        match self {
            Gender::Male => LocalizedArg::Str("male"),
            Gender::Female => LocalizedArg::Str("female"),
            Gender::Other => LocalizedArg::Str("other"),
        }
    }
}

fn main() {
    let mut application = Application::new();

    let username = LocalState::new("Emma".to_string());
    let photo_count = LocalState::new(3.0);
    let gender = LocalState::new(Gender::Female);

    application.set_scene(Window::new(
        "LocalizedString - Carbide",
        Dimension::new(400.0, 400.0),
        VStack::new((
            Text::new(
                LocalizedString::new("shared-photos")
                    .arg("userName", username.clone())
                    .arg("userGender", gender.clone())
                    .arg("photoCount", photo_count.clone())
            ).padding(30.0)
                .border()
                .color(EnvironmentColor::Accent),
            VStack::new((
                TextInput::new(username)
                    .label("Username:"),
                PopUpButton::new(gender, vec![
                    Gender::Male,
                    Gender::Female,
                    Gender::Other,
                ]).label("Gender:"),
                Slider::new(photo_count, 1.0, 10.0)
                    .step(1.0)
                    .label("Photo count:")
            ))
        )).spacing(30.0)
            .padding(80.0)
    ).close_application_on_window_close());

    application.launch();
}