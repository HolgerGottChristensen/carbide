use std::time::Duration;

use chrono::Local;
use icu::locid::locale;

use carbide_controls::{ControlsExt, PopUpButton, Slider};
use carbide_core::asynchronous::Timer;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{GlobalState, LocalState, Map1, State, ValueState};
use carbide_core::widget::{EdgeInsets, Text, VStack, WidgetExt};
use carbide_fluent::{DateStyle, LocalizedDateTime, LocalizedNumber, LocalizedString, NumberGrouping, NumberNotation, NumberStyle, RoundingMode, TimeStyle, TimezoneStyle};
use carbide_fluent::LocaleExt;
use carbide_wgpu::{Application, Window};

fn main() {
    let mut application = Application::new();

    let locale = LocalState::new(locale!("en"));

    let number = LocalState::new(1234.567);
    let style = LocalState::new(NumberStyle::default());
    let notation = LocalState::new(NumberNotation::default());
    let grouping = LocalState::new(NumberGrouping::default());
    let rounding = LocalState::new(RoundingMode::default());
    let minimum_integer_digits = LocalState::new(2usize);
    let minimum_fraction_digits = LocalState::new(2usize);
    let maximum_fraction_digits = LocalState::new(2usize);
    let minimum_significant_digits = LocalState::new(2usize);
    let maximum_significant_digits = LocalState::new(2usize);

    let text = Text::new(
        LocalizedNumber::new(number.clone())
            .style(style.clone())
            .notation(notation.clone())
            .use_grouping(grouping.clone())
            .minimum_integer_digits(minimum_integer_digits.clone())
            .minimum_fraction_digits(minimum_fraction_digits.clone())
            .maximum_fraction_digits(maximum_fraction_digits.clone())
            .minimum_significant_digits(minimum_significant_digits.clone())
            .maximum_significant_digits(maximum_significant_digits.clone())
            .rounding_mode(rounding.clone())
    )
        .padding(30.0)
        .border()
        .color(EnvironmentColor::Accent);

    application.set_scene(Window::new(
        "LocalizedNumber - Carbide",
        Dimension::new(400.0, 600.0),
        VStack::new((
            text,
            VStack::new((
                Slider::new(number.clone(), 0.0, 100000.0).label(LocalizedString::new("number")),
                PopUpButton::new(style.clone(), vec![
                    NumberStyle::Decimal,
                    NumberStyle::Currency,
                    NumberStyle::Percent,
                ]).label(LocalizedString::new("style")),
                PopUpButton::new(notation.clone(), vec![
                    NumberNotation::Standard,
                    NumberNotation::Scientific,
                    NumberNotation::Engineering,
                ]).label(LocalizedString::new("notation")),
                PopUpButton::new(grouping.clone(), vec![
                    NumberGrouping::Auto,
                    NumberGrouping::Always,
                    NumberGrouping::Min2,
                    NumberGrouping::Never,
                ]).label(LocalizedString::new("grouping")),
                PopUpButton::new(rounding.clone(), vec![
                    RoundingMode::Ceil,
                    RoundingMode::Floor,
                    RoundingMode::Expand,
                    RoundingMode::Trunc,
                    RoundingMode::HalfCeil,
                    RoundingMode::HalfFloor,
                    RoundingMode::HalfExpand,
                    RoundingMode::HalfTrunc,
                    RoundingMode::HalfEven,
                ]).label(LocalizedString::new("rounding")),
                Slider::new(minimum_integer_digits, 0usize, 10usize).label(LocalizedString::new("minimum_integer_digits")),
                Slider::new(minimum_fraction_digits, 0usize, 10usize).label(LocalizedString::new("minimum_fraction_digits")),
                Slider::new(maximum_fraction_digits, 0usize, 10usize).label(LocalizedString::new("maximum_fraction_digits")),
                Slider::new(minimum_significant_digits, 0usize, 10usize).label(LocalizedString::new("minimum_significant_digits")),
                Slider::new(maximum_significant_digits, 0usize, 10usize).label(LocalizedString::new("maximum_significant_digits")),
            )).spacing(15.0),
            PopUpButton::new(locale.clone(), vec![
                locale!("en"),
                locale!("da"),
                locale!("ja"),
                locale!("de"),
                locale!("en-US"),
                locale!("en-GB"),
            ]).label(LocalizedString::new("locale"))
        )).spacing(30.0)
            .padding(EdgeInsets::vertical_horizontal(50.0, 30.0))
            .locale(locale)
    ).close_application_on_window_close());

    application.launch();
}