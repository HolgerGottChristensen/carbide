use chrono::Local;
use icu::locid::locale;

use carbide_controls::{ControlsExt};
use carbide_controls::picker::{MenuStyle, Picker};
use carbide_core::asynchronous::Timer;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{GlobalState, LocalState, State};
use carbide_core::widget::{Text, VStack, WidgetExt};
use carbide_core::time::*;
use carbide_fluent::{DateStyle, LocalizedDateTime, LocalizedString, TimeStyle, TimezoneStyle};
use carbide_fluent::LocaleExt;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let locale = LocalState::new(locale!("en"));

    let time = GlobalState::new(Local::now().fixed_offset());
    let time_clone = time.clone();

    let date_style = LocalState::new(DateStyle::Full);
    let time_style = LocalState::new(TimeStyle::Full);
    let timezone_style = LocalState::new(TimezoneStyle::Hidden);

    let _t = Timer::new(move || {
        time_clone.clone().set_value(Local::now().fixed_offset());
    }).interval(Duration::from_secs_f64(0.1))
        .repeat()
        .start();

    let text = Text::new(
        LocalizedDateTime::new(time)
            .date_style(date_style.clone())
            .time_style(time_style.clone())
            .timezone_style(timezone_style.clone())
    )
        .padding(30.0)
        .border()
        .color(EnvironmentColor::Accent);

    application.set_scene(Window::new(
        "LocalizedDateTime - Carbide",
        Dimension::new(400.0, 400.0),
        VStack::new((
            text,
            VStack::new((
                Picker::new(LocalizedString::new("datestyle"), date_style.clone(), (
                    Text::new("Full").tag(DateStyle::Full),
                    Text::new("Long").tag(DateStyle::Long),
                    Text::new("Medium").tag(DateStyle::Medium),
                    Text::new("Short").tag(DateStyle::Short),
                    Text::new("Hidden").tag(DateStyle::Hidden),
                )),
                Picker::new(LocalizedString::new("timestyle"), time_style.clone(), (
                    Text::new("Full").tag(TimeStyle::Full),
                    Text::new("Long").tag(TimeStyle::Long),
                    Text::new("Medium").tag(TimeStyle::Medium),
                    Text::new("Short").tag(TimeStyle::Short),
                    Text::new("Hidden").tag(TimeStyle::Hidden),
                )),
                Picker::new(LocalizedString::new("timezonestyle"), timezone_style.clone(), (
                    Text::new("Hidden").tag(TimezoneStyle::Hidden),
                    Text::new("LocalizedGmt").tag(TimezoneStyle::LocalizedGmt),
                )),
            )),
            Picker::new(LocalizedString::new("locale"), locale.clone(), (
                Text::new("en").tag(locale!("en")),
                Text::new("da").tag(locale!("da")),
                Text::new("ja").tag(locale!("ja")),
                Text::new("de").tag(locale!("de")),
                Text::new("en-US").tag(locale!("en-US")),
                Text::new("en-GB").tag(locale!("en-GB")),
            )),
        )).spacing(30.0)
            .padding(80.0)
            .locale(locale)
            .picker_style(MenuStyle)
    ));

    application.launch();
}