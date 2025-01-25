use crate::draw::Color;
use crate::environment::{Environment, EnvironmentKey, EnvironmentKeyable};
use crate::render::Style;
use crate::state::*;

#[derive(Hash, Eq, PartialEq, Clone, Debug, Copy)]
pub enum EnvironmentColor {
    Blue,
    Green,
    Indigo,
    Orange,
    Pink,
    Purple,
    Red,
    Teal,
    Yellow,
    Gray,
    Gray2,
    Gray3,
    Gray4,
    Gray5,
    Gray6,
    Label,
    SecondaryLabel,
    TertiaryLabel,
    QuaternaryLabel,
    SystemFill,
    SecondarySystemFill,
    TertiarySystemFill,
    QuaternarySystemFill,
    PlaceholderText,
    SystemBackground,
    SecondarySystemBackground,
    TertiarySystemBackground,
    Separator,
    OpaqueSeparator,
    Link,
    DarkText,
    LightText,
    Accent,

    // Material colors theme
    UltraThick,
    Thick,
    Regular,
    Thin,
    UltraThin,

    // Material colors theme light
    UltraThickLight,
    ThickLight,
    RegularLight,
    ThinLight,
    UltraThinLight,

    // Material colors theme dark
    UltraThickDark,
    ThickDark,
    RegularDark,
    ThinDark,
    UltraThinDark,
}

impl EnvironmentKeyable for EnvironmentColor {
    type Output = Color;

    fn get(&self, stack: &Environment) -> Option<Self::Output> {
        match self {
            EnvironmentColor::Blue => stack.get::<EnvironmentColorBlue>().cloned(),
            EnvironmentColor::Green => stack.get::<EnvironmentColorGreen>().cloned(),
            EnvironmentColor::Indigo => stack.get::<EnvironmentColorIndigo>().cloned(),
            EnvironmentColor::Orange => stack.get::<EnvironmentColorOrange>().cloned(),
            EnvironmentColor::Pink => stack.get::<EnvironmentColorPink>().cloned(),
            EnvironmentColor::Purple => stack.get::<EnvironmentColorPurple>().cloned(),
            EnvironmentColor::Red => stack.get::<EnvironmentColorRed>().cloned(),
            EnvironmentColor::Teal => stack.get::<EnvironmentColorTeal>().cloned(),
            EnvironmentColor::Yellow => stack.get::<EnvironmentColorYellow>().cloned(),
            EnvironmentColor::Gray => stack.get::<EnvironmentColorGray>().cloned(),
            EnvironmentColor::Gray2 => stack.get::<EnvironmentColorGray2>().cloned(),
            EnvironmentColor::Gray3 => stack.get::<EnvironmentColorGray3>().cloned(),
            EnvironmentColor::Gray4 => stack.get::<EnvironmentColorGray4>().cloned(),
            EnvironmentColor::Gray5 => stack.get::<EnvironmentColorGray5>().cloned(),
            EnvironmentColor::Gray6 => stack.get::<EnvironmentColorGray6>().cloned(),
            EnvironmentColor::Label => stack.get::<EnvironmentColorLabel>().cloned(),
            EnvironmentColor::SecondaryLabel => stack.get::<EnvironmentColorSecondaryLabel>().cloned(),
            EnvironmentColor::TertiaryLabel => stack.get::<EnvironmentColorTertiaryLabel>().cloned(),
            EnvironmentColor::QuaternaryLabel => stack.get::<EnvironmentColorQuaternaryLabel>().cloned(),
            EnvironmentColor::SystemFill => stack.get::<EnvironmentColorSystemFill>().cloned(),
            EnvironmentColor::SecondarySystemFill => stack.get::<EnvironmentColorSecondarySystemFill>().cloned(),
            EnvironmentColor::TertiarySystemFill => stack.get::<EnvironmentColorTertiarySystemFill>().cloned(),
            EnvironmentColor::QuaternarySystemFill => stack.get::<EnvironmentColorQuaternarySystemFill>().cloned(),
            EnvironmentColor::PlaceholderText => stack.get::<EnvironmentColorPlaceholderText>().cloned(),
            EnvironmentColor::SystemBackground => stack.get::<EnvironmentColorSystemBackground>().cloned(),
            EnvironmentColor::SecondarySystemBackground => stack.get::<EnvironmentColorSecondarySystemBackground>().cloned(),
            EnvironmentColor::TertiarySystemBackground => stack.get::<EnvironmentColorTertiarySystemBackground>().cloned(),
            EnvironmentColor::Separator => stack.get::<EnvironmentColorSeparator>().cloned(),
            EnvironmentColor::OpaqueSeparator => stack.get::<EnvironmentColorOpaqueSeparator>().cloned(),
            EnvironmentColor::Link => stack.get::<EnvironmentColorLink>().cloned(),
            EnvironmentColor::DarkText => stack.get::<EnvironmentColorDarkText>().cloned(),
            EnvironmentColor::LightText => stack.get::<EnvironmentColorLightText>().cloned(),
            EnvironmentColor::Accent => stack.get::<EnvironmentColorAccent>().cloned(),
            EnvironmentColor::UltraThick => stack.get::<EnvironmentColorUltraThick>().cloned(),
            EnvironmentColor::Thick => stack.get::<EnvironmentColorThick>().cloned(),
            EnvironmentColor::Regular => stack.get::<EnvironmentColorRegular>().cloned(),
            EnvironmentColor::Thin => stack.get::<EnvironmentColorThin>().cloned(),
            EnvironmentColor::UltraThin => stack.get::<EnvironmentColorUltraThin>().cloned(),
            EnvironmentColor::UltraThickLight => stack.get::<EnvironmentColorUltraThickLight>().cloned(),
            EnvironmentColor::ThickLight => stack.get::<EnvironmentColorThickLight>().cloned(),
            EnvironmentColor::RegularLight => stack.get::<EnvironmentColorRegularLight>().cloned(),
            EnvironmentColor::ThinLight => stack.get::<EnvironmentColorThinLight>().cloned(),
            EnvironmentColor::UltraThinLight => stack.get::<EnvironmentColorUltraThinLight>().cloned(),
            EnvironmentColor::UltraThickDark => stack.get::<EnvironmentColorUltraThickDark>().cloned(),
            EnvironmentColor::ThickDark => stack.get::<EnvironmentColorThickDark>().cloned(),
            EnvironmentColor::RegularDark => stack.get::<EnvironmentColorRegularDark>().cloned(),
            EnvironmentColor::ThinDark => stack.get::<EnvironmentColorThinDark>().cloned(),
            EnvironmentColor::UltraThinDark => stack.get::<EnvironmentColorUltraThinDark>().cloned(),
        }
    }

    fn with(&self, value: &Self::Output, stack: &mut Environment, f: impl FnOnce(&mut Environment)) {
        match self {
            EnvironmentColor::Blue => stack.with::<EnvironmentColorBlue>(value, f),
            EnvironmentColor::Green => stack.with::<EnvironmentColorGreen>(value, f),
            EnvironmentColor::Indigo => stack.with::<EnvironmentColorIndigo>(value, f),
            EnvironmentColor::Orange => stack.with::<EnvironmentColorOrange>(value, f),
            EnvironmentColor::Pink => stack.with::<EnvironmentColorPink>(value, f),
            EnvironmentColor::Purple => stack.with::<EnvironmentColorPurple>(value, f),
            EnvironmentColor::Red => stack.with::<EnvironmentColorRed>(value, f),
            EnvironmentColor::Teal => stack.with::<EnvironmentColorTeal>(value, f),
            EnvironmentColor::Yellow => stack.with::<EnvironmentColorYellow>(value, f),
            EnvironmentColor::Gray => stack.with::<EnvironmentColorGray>(value, f),
            EnvironmentColor::Gray2 => stack.with::<EnvironmentColorGray2>(value, f),
            EnvironmentColor::Gray3 => stack.with::<EnvironmentColorGray3>(value, f),
            EnvironmentColor::Gray4 => stack.with::<EnvironmentColorGray4>(value, f),
            EnvironmentColor::Gray5 => stack.with::<EnvironmentColorGray5>(value, f),
            EnvironmentColor::Gray6 => stack.with::<EnvironmentColorGray6>(value, f),
            EnvironmentColor::Label => stack.with::<EnvironmentColorLabel>(value, f),
            EnvironmentColor::SecondaryLabel => stack.with::<EnvironmentColorSecondaryLabel>(value, f),
            EnvironmentColor::TertiaryLabel => stack.with::<EnvironmentColorTertiaryLabel>(value, f),
            EnvironmentColor::QuaternaryLabel => stack.with::<EnvironmentColorQuaternaryLabel>(value, f),
            EnvironmentColor::SystemFill => stack.with::<EnvironmentColorSystemFill>(value, f),
            EnvironmentColor::SecondarySystemFill => stack.with::<EnvironmentColorSecondarySystemFill>(value, f),
            EnvironmentColor::TertiarySystemFill => stack.with::<EnvironmentColorTertiarySystemFill>(value, f),
            EnvironmentColor::QuaternarySystemFill => stack.with::<EnvironmentColorQuaternarySystemFill>(value, f),
            EnvironmentColor::PlaceholderText => stack.with::<EnvironmentColorPlaceholderText>(value, f),
            EnvironmentColor::SystemBackground => stack.with::<EnvironmentColorSystemBackground>(value, f),
            EnvironmentColor::SecondarySystemBackground => stack.with::<EnvironmentColorSecondarySystemBackground>(value, f),
            EnvironmentColor::TertiarySystemBackground => stack.with::<EnvironmentColorTertiarySystemBackground>(value, f),
            EnvironmentColor::Separator => stack.with::<EnvironmentColorSeparator>(value, f),
            EnvironmentColor::OpaqueSeparator => stack.with::<EnvironmentColorOpaqueSeparator>(value, f),
            EnvironmentColor::Link => stack.with::<EnvironmentColorLink>(value, f),
            EnvironmentColor::DarkText => stack.with::<EnvironmentColorDarkText>(value, f),
            EnvironmentColor::LightText => stack.with::<EnvironmentColorLightText>(value, f),
            EnvironmentColor::Accent => stack.with::<EnvironmentColorAccent>(value, f),
            EnvironmentColor::UltraThick => stack.with::<EnvironmentColorUltraThick>(value, f),
            EnvironmentColor::Thick => stack.with::<EnvironmentColorThick>(value, f),
            EnvironmentColor::Regular => stack.with::<EnvironmentColorRegular>(value, f),
            EnvironmentColor::Thin => stack.with::<EnvironmentColorThin>(value, f),
            EnvironmentColor::UltraThin => stack.with::<EnvironmentColorUltraThin>(value, f),
            EnvironmentColor::UltraThickLight => stack.with::<EnvironmentColorUltraThickLight>(value, f),
            EnvironmentColor::ThickLight => stack.with::<EnvironmentColorThickLight>(value, f),
            EnvironmentColor::RegularLight => stack.with::<EnvironmentColorRegularLight>(value, f),
            EnvironmentColor::ThinLight => stack.with::<EnvironmentColorThinLight>(value, f),
            EnvironmentColor::UltraThinLight => stack.with::<EnvironmentColorUltraThinLight>(value, f),
            EnvironmentColor::UltraThickDark => stack.with::<EnvironmentColorUltraThickDark>(value, f),
            EnvironmentColor::ThickDark => stack.with::<EnvironmentColorThickDark>(value, f),
            EnvironmentColor::RegularDark => stack.with::<EnvironmentColorRegularDark>(value, f),
            EnvironmentColor::ThinDark => stack.with::<EnvironmentColorThinDark>(value, f),
            EnvironmentColor::UltraThinDark => stack.with::<EnvironmentColorUltraThinDark>(value, f),
        }
    }
}


impl EnvironmentColor {
    pub fn style(&self) -> impl ReadState<T=Style> {
        <EnvironmentColor as IntoReadState<Style>>::into_read_state(self.clone())
    }
}

impl Default for EnvironmentColor {
    fn default() -> Self {
        EnvironmentColor::Blue
    }
}

// ---------------------------------------------------
//  Conversion implementations
// ---------------------------------------------------

impl ConvertIntoRead<Color> for EnvironmentColor {
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&mut Environment, &EnvironmentColor)->Color, EnvironmentColor, Color, G>;

    fn convert<F: AnyReadState<T=EnvironmentColor> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| {
            value.get(env).unwrap_or_default()
        })
    }
}

impl ConvertIntoRead<Style> for EnvironmentColor {
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&mut Environment, &EnvironmentColor)->Style, EnvironmentColor, Style, G>;

    fn convert<F: AnyReadState<T=EnvironmentColor> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| {
            Style::Color(value.get(env).unwrap_or_default())
        })
    }
}

pub trait IntoColorReadState {
    type Output: ReadState<T=Color>;
    fn color(self) -> Self::Output;
}

impl<T> IntoColorReadState for T where T: IntoReadState<Color> {
    type Output = T::Output;

    fn color(self) -> Self::Output {
        self.into_read_state()
    }
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorBlue;
impl EnvironmentKey for EnvironmentColorBlue {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGreen;
impl EnvironmentKey for EnvironmentColorGreen {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorIndigo;
impl EnvironmentKey for EnvironmentColorIndigo {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorOrange;
impl EnvironmentKey for EnvironmentColorOrange {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorPink;
impl EnvironmentKey for EnvironmentColorPink {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorPurple;
impl EnvironmentKey for EnvironmentColorPurple {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorRed;
impl EnvironmentKey for EnvironmentColorRed {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorTeal;
impl EnvironmentKey for EnvironmentColorTeal {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorYellow;
impl EnvironmentKey for EnvironmentColorYellow {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray;
impl EnvironmentKey for EnvironmentColorGray {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray2;
impl EnvironmentKey for EnvironmentColorGray2 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray3;
impl EnvironmentKey for EnvironmentColorGray3 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray4;
impl EnvironmentKey for EnvironmentColorGray4 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray5;
impl EnvironmentKey for EnvironmentColorGray5 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray6;
impl EnvironmentKey for EnvironmentColorGray6 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct EnvironmentColorLabel;
impl EnvironmentKey for EnvironmentColorLabel {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSecondaryLabel;
impl EnvironmentKey for EnvironmentColorSecondaryLabel {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorTertiaryLabel;
impl EnvironmentKey for EnvironmentColorTertiaryLabel {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorQuaternaryLabel;
impl EnvironmentKey for EnvironmentColorQuaternaryLabel {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSystemFill;
impl EnvironmentKey for EnvironmentColorSystemFill {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSecondarySystemFill;
impl EnvironmentKey for EnvironmentColorSecondarySystemFill {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorTertiarySystemFill;
impl EnvironmentKey for EnvironmentColorTertiarySystemFill {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorQuaternarySystemFill;
impl EnvironmentKey for EnvironmentColorQuaternarySystemFill {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorPlaceholderText;
impl EnvironmentKey for EnvironmentColorPlaceholderText {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSystemBackground;
impl EnvironmentKey for EnvironmentColorSystemBackground {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSecondarySystemBackground;
impl EnvironmentKey for EnvironmentColorSecondarySystemBackground {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorTertiarySystemBackground;
impl EnvironmentKey for EnvironmentColorTertiarySystemBackground {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSeparator;
impl EnvironmentKey for EnvironmentColorSeparator {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorOpaqueSeparator;
impl EnvironmentKey for EnvironmentColorOpaqueSeparator {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorLink;
impl EnvironmentKey for EnvironmentColorLink {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorDarkText;
impl EnvironmentKey for EnvironmentColorDarkText {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorLightText;
impl EnvironmentKey for EnvironmentColorLightText {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct EnvironmentColorAccent;
impl EnvironmentKey for EnvironmentColorAccent {
    type Value = Color;
}

// Material colors theme
#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThick;
impl EnvironmentKey for EnvironmentColorUltraThick {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThick;
impl EnvironmentKey for EnvironmentColorThick {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorRegular;
impl EnvironmentKey for EnvironmentColorRegular {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThin;
impl EnvironmentKey for EnvironmentColorThin {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThin;
impl EnvironmentKey for EnvironmentColorUltraThin {
    type Value = Color;
}

// Material colors theme light
#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThickLight;
impl EnvironmentKey for EnvironmentColorUltraThickLight {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThickLight;
impl EnvironmentKey for EnvironmentColorThickLight {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorRegularLight;
impl EnvironmentKey for EnvironmentColorRegularLight {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThinLight;
impl EnvironmentKey for EnvironmentColorThinLight {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThinLight;
impl EnvironmentKey for EnvironmentColorUltraThinLight {
    type Value = Color;
}

// Material colors theme dark
#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThickDark;
impl EnvironmentKey for EnvironmentColorUltraThickDark {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThickDark;
impl EnvironmentKey for EnvironmentColorThickDark {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorRegularDark;
impl EnvironmentKey for EnvironmentColorRegularDark {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThinDark;
impl EnvironmentKey for EnvironmentColorThinDark {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThinDark;
impl EnvironmentKey for EnvironmentColorUltraThinDark {
    type Value = Color;
}