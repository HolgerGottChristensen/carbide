use icu::locid::locale;

use carbide_controls::{ControlsExt};
use carbide_controls::picker::Picker;
use carbide_controls::slider::Slider;
use carbide_core::draw::Dimension;
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::LocalState;
use carbide_core::widget::{EdgeInsets, Text, VStack, WidgetExt};
use carbide_fluent::{LocalizedNumber, LocalizedString, NumberGrouping, NumberNotation, NumberStyle, RoundingMode};
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
                Slider::new(number.clone(), 0.0, 100000.0)
                    .label(LocalizedString::new("number")),
                Picker::new(LocalizedString::new("style"), style.clone(), (
                    Text::new("Decimal").tag(NumberStyle::Decimal),
                    Text::new("Currency").tag(NumberStyle::Currency),
                    Text::new("Percent").tag(NumberStyle::Percent),
                )),
                Picker::new(LocalizedString::new("notation"), notation.clone(), (
                    Text::new("Standard").tag(NumberNotation::Standard),
                    Text::new("Scientific").tag(NumberNotation::Scientific),
                    Text::new("Engineering").tag(NumberNotation::Engineering),
                )),
                Picker::new(LocalizedString::new("grouping"), grouping.clone(), (
                    Text::new("Auto").tag(NumberGrouping::Auto),
                    Text::new("Always").tag(NumberGrouping::Always),
                    Text::new("Min2").tag(NumberGrouping::Min2),
                    Text::new("Never").tag(NumberGrouping::Never),
                )),
                Picker::new(LocalizedString::new("rounding"), rounding.clone(), (
                    Text::new("Ceil").tag(RoundingMode::Ceil),
                    Text::new("Floor").tag(RoundingMode::Floor),
                    Text::new("Expand").tag(RoundingMode::Expand),
                    Text::new("Trunc").tag(RoundingMode::Trunc),
                    Text::new("HalfCeil").tag(RoundingMode::HalfCeil),
                    Text::new("HalfFloor").tag(RoundingMode::HalfFloor),
                    Text::new("HalfExpand").tag(RoundingMode::HalfExpand),
                    Text::new("HalfTrunc").tag(RoundingMode::HalfTrunc),
                    Text::new("HalfEven").tag(RoundingMode::HalfEven),
                )),
                Slider::new(minimum_integer_digits, 0usize, 10usize)
                    .label(LocalizedString::new("minimum_integer_digits")),
                Slider::new(minimum_fraction_digits, 0usize, 10usize)
                    .label(LocalizedString::new("minimum_fraction_digits")),
                Slider::new(maximum_fraction_digits, 0usize, 10usize)
                    .label(LocalizedString::new("maximum_fraction_digits")),
                Slider::new(minimum_significant_digits, 0usize, 10usize)
                    .label(LocalizedString::new("minimum_significant_digits")),
                Slider::new(maximum_significant_digits, 0usize, 10usize)
                    .label(LocalizedString::new("maximum_significant_digits")),
            )).spacing(15.0),
            Picker::new(LocalizedString::new("locale"), locale.clone(), (
                Text::new("en").tag(locale!("en")),
                Text::new("da").tag(locale!("da")),
                Text::new("ja").tag(locale!("ja")),
                Text::new("de").tag(locale!("de")),
                Text::new("en-US").tag(locale!("en-US")),
                Text::new("en-GB").tag(locale!("en-GB")),
            )),
        )).spacing(30.0)
            .padding(EdgeInsets::vertical_horizontal(50.0, 30.0))
            .locale(locale)
    ));

    application.launch();
}