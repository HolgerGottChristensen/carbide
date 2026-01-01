use icu::locid::locale;

use carbide_controls::{ControlsExt, TextInput};
use carbide_controls::picker::{MenuStyle, Picker};
use carbide_controls::slider::Slider;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{LocalState, StateValue};
use carbide_core::widget::{Text, VStack, WidgetExt};
use carbide_fluent::{Arg, Localizable, LocalizedArg, LocalizedString};
use carbide_fluent::LocaleExt;
use carbide_wgpu::{Application, Window};

#[derive(Clone, Debug, PartialEq, Eq, Hash, StateValue)]
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

impl Localizable for Gender {
    fn get(&self) -> &str {
        match self {
            Gender::Male => "gender.male",
            Gender::Female => "gender.female",
            Gender::Other => "gender.other",
        }
    }
}

fn main() {
    let mut application = Application::new();

    let locale = LocalState::new(locale!("en"));

    let username = LocalState::new("Emma".to_string());
    let photo_count = LocalState::new(3);
    let gender = LocalState::new(Gender::Female);

    let text = Text::new(
        LocalizedString::new("shared_photos")
            .arg("userName", username.clone())
            .arg("userGender", gender.clone())
            .arg("photoCount", photo_count.clone())
    ).padding(30.0)
        .border()
        .color(EnvironmentColor::Accent);

    application.set_scene(Window::new(
        "LocalizedString - Carbide",
        Dimension::new(400.0, 400.0),
        VStack::new((
            text,
            VStack::new((
                TextInput::new(username)
                    .label(LocalizedString::new("username")),
                Picker::new(LocalizedString::new("gender"), gender.clone(), (
                    Text::new(LocalizedString::new(Gender::Male)).tag(Gender::Male),
                    Text::new(LocalizedString::new(Gender::Female)).tag(Gender::Female),
                    Text::new(LocalizedString::new(Gender::Other)).tag(Gender::Other),
                )),
                Slider::new(photo_count, 1, 10)
                    .label(LocalizedString::new("photo_count")),
                Picker::new(LocalizedString::new("locale"), locale.clone(), (
                    Text::new("en").tag(locale!("en")),
                    Text::new("da").tag(locale!("da")),
                )),
            ))
        )).spacing(30.0)
            .padding(80.0)
            .locale(locale)
            .picker_style(MenuStyle)
    ));

    application.launch();
}