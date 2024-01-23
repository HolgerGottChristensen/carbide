use std::time::Duration;

use chrono::Local;
use icu::locid::locale;

use carbide_controls::{ControlsExt, PopUpButton};
use carbide_core::asynchronous::Timer;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{GlobalState, LocalState, State, ValueState};
use carbide_core::widget::{Text, VStack, WidgetExt};
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
                PopUpButton::new(date_style.clone(), vec![
                    DateStyle::Full,
                    DateStyle::Long,
                    DateStyle::Medium,
                    DateStyle::Short,
                    DateStyle::Hidden
                ]).label(LocalizedString::new("datestyle")),
                PopUpButton::new(time_style.clone(), vec![
                    TimeStyle::Full,
                    TimeStyle::Long,
                    TimeStyle::Medium,
                    TimeStyle::Short,
                    TimeStyle::Hidden
                ]).label(LocalizedString::new("timestyle")),
                PopUpButton::new(timezone_style.clone(), vec![
                    TimezoneStyle::Hidden,
                    TimezoneStyle::LocalizedGmt,
                ]).label(LocalizedString::new("timezonestyle")),
            )),
            PopUpButton::new(locale.clone(), vec![
                locale!("en"),
                locale!("da"),
                locale!("ja"),
                locale!("de"),
                locale!("en-US"),
                locale!("en-GB"),
            ]).label(LocalizedString::new("locale"))
        )).spacing(30.0)
            .padding(80.0)
            .locale(locale)
    ).close_application_on_window_close());

    application.launch();
}