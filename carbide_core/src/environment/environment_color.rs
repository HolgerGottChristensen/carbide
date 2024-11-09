use crate::draw::Color;
use crate::environment::{EnvironmentStack, Key, Keyable};
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

impl Keyable for EnvironmentColor {
    type Output = Color;

    fn get(&self, stack: &EnvironmentStack) -> Self::Output {
        match self {
            EnvironmentColor::Blue => stack.get::<EnvironmentColorBlue>().cloned().unwrap_or_default(),
            EnvironmentColor::Green => stack.get::<EnvironmentColorGreen>().cloned().unwrap_or_default(),
            EnvironmentColor::Indigo => stack.get::<EnvironmentColorIndigo>().cloned().unwrap_or_default(),
            EnvironmentColor::Orange => stack.get::<EnvironmentColorOrange>().cloned().unwrap_or_default(),
            EnvironmentColor::Pink => stack.get::<EnvironmentColorPink>().cloned().unwrap_or_default(),
            EnvironmentColor::Purple => stack.get::<EnvironmentColorPurple>().cloned().unwrap_or_default(),
            EnvironmentColor::Red => stack.get::<EnvironmentColorRed>().cloned().unwrap_or_default(),
            EnvironmentColor::Teal => stack.get::<EnvironmentColorTeal>().cloned().unwrap_or_default(),
            EnvironmentColor::Yellow => stack.get::<EnvironmentColorYellow>().cloned().unwrap_or_default(),
            EnvironmentColor::Gray => stack.get::<EnvironmentColorGray>().cloned().unwrap_or_default(),
            EnvironmentColor::Gray2 => stack.get::<EnvironmentColorGray2>().cloned().unwrap_or_default(),
            EnvironmentColor::Gray3 => stack.get::<EnvironmentColorGray3>().cloned().unwrap_or_default(),
            EnvironmentColor::Gray4 => stack.get::<EnvironmentColorGray4>().cloned().unwrap_or_default(),
            EnvironmentColor::Gray5 => stack.get::<EnvironmentColorGray5>().cloned().unwrap_or_default(),
            EnvironmentColor::Gray6 => stack.get::<EnvironmentColorGray6>().cloned().unwrap_or_default(),
            EnvironmentColor::Label => stack.get::<EnvironmentColorLabel>().cloned().unwrap_or_default(),
            EnvironmentColor::SecondaryLabel => stack.get::<EnvironmentColorSecondaryLabel>().cloned().unwrap_or_default(),
            EnvironmentColor::TertiaryLabel => stack.get::<EnvironmentColorTertiaryLabel>().cloned().unwrap_or_default(),
            EnvironmentColor::QuaternaryLabel => stack.get::<EnvironmentColorQuaternaryLabel>().cloned().unwrap_or_default(),
            EnvironmentColor::SystemFill => stack.get::<EnvironmentColorSystemFill>().cloned().unwrap_or_default(),
            EnvironmentColor::SecondarySystemFill => stack.get::<EnvironmentColorSecondarySystemFill>().cloned().unwrap_or_default(),
            EnvironmentColor::TertiarySystemFill => stack.get::<EnvironmentColorTertiarySystemFill>().cloned().unwrap_or_default(),
            EnvironmentColor::QuaternarySystemFill => stack.get::<EnvironmentColorQuaternarySystemFill>().cloned().unwrap_or_default(),
            EnvironmentColor::PlaceholderText => stack.get::<EnvironmentColorPlaceholderText>().cloned().unwrap_or_default(),
            EnvironmentColor::SystemBackground => stack.get::<EnvironmentColorSystemBackground>().cloned().unwrap_or_default(),
            EnvironmentColor::SecondarySystemBackground => stack.get::<EnvironmentColorSecondarySystemBackground>().cloned().unwrap_or_default(),
            EnvironmentColor::TertiarySystemBackground => stack.get::<EnvironmentColorTertiarySystemBackground>().cloned().unwrap_or_default(),
            EnvironmentColor::Separator => stack.get::<EnvironmentColorSeparator>().cloned().unwrap_or_default(),
            EnvironmentColor::OpaqueSeparator => stack.get::<EnvironmentColorOpaqueSeparator>().cloned().unwrap_or_default(),
            EnvironmentColor::Link => stack.get::<EnvironmentColorLink>().cloned().unwrap_or_default(),
            EnvironmentColor::DarkText => stack.get::<EnvironmentColorDarkText>().cloned().unwrap_or_default(),
            EnvironmentColor::LightText => stack.get::<EnvironmentColorLightText>().cloned().unwrap_or_default(),
            EnvironmentColor::Accent => stack.get::<EnvironmentColorAccent>().cloned().unwrap_or_default(),
            EnvironmentColor::UltraThick => stack.get::<EnvironmentColorUltraThick>().cloned().unwrap_or_default(),
            EnvironmentColor::Thick => stack.get::<EnvironmentColorThick>().cloned().unwrap_or_default(),
            EnvironmentColor::Regular => stack.get::<EnvironmentColorRegular>().cloned().unwrap_or_default(),
            EnvironmentColor::Thin => stack.get::<EnvironmentColorThin>().cloned().unwrap_or_default(),
            EnvironmentColor::UltraThin => stack.get::<EnvironmentColorUltraThin>().cloned().unwrap_or_default(),
            EnvironmentColor::UltraThickLight => stack.get::<EnvironmentColorUltraThickLight>().cloned().unwrap_or_default(),
            EnvironmentColor::ThickLight => stack.get::<EnvironmentColorThickLight>().cloned().unwrap_or_default(),
            EnvironmentColor::RegularLight => stack.get::<EnvironmentColorRegularLight>().cloned().unwrap_or_default(),
            EnvironmentColor::ThinLight => stack.get::<EnvironmentColorThinLight>().cloned().unwrap_or_default(),
            EnvironmentColor::UltraThinLight => stack.get::<EnvironmentColorUltraThinLight>().cloned().unwrap_or_default(),
            EnvironmentColor::UltraThickDark => stack.get::<EnvironmentColorUltraThickDark>().cloned().unwrap_or_default(),
            EnvironmentColor::ThickDark => stack.get::<EnvironmentColorThickDark>().cloned().unwrap_or_default(),
            EnvironmentColor::RegularDark => stack.get::<EnvironmentColorRegularDark>().cloned().unwrap_or_default(),
            EnvironmentColor::ThinDark => stack.get::<EnvironmentColorThinDark>().cloned().unwrap_or_default(),
            EnvironmentColor::UltraThinDark => stack.get::<EnvironmentColorUltraThinDark>().cloned().unwrap_or_default(),
        }
    }

    fn with(&self, value: &Self::Output, stack: &mut EnvironmentStack, f: impl FnOnce(&mut EnvironmentStack)) {
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
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&EnvironmentStack, &EnvironmentColor)->Color, EnvironmentColor, Color, G>;

    fn convert<F: AnyReadState<T=EnvironmentColor> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| {
            value.get(env)
        })
    }
}

impl ConvertIntoRead<Style> for EnvironmentColor {
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&EnvironmentStack, &EnvironmentColor)->Style, EnvironmentColor, Style, G>;

    fn convert<F: AnyReadState<T=EnvironmentColor> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| {
            Style::Color(value.get(env))
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
impl Key for EnvironmentColorBlue {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGreen;
impl Key for EnvironmentColorGreen {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorIndigo;
impl Key for EnvironmentColorIndigo {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorOrange;
impl Key for EnvironmentColorOrange {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorPink;
impl Key for EnvironmentColorPink {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorPurple;
impl Key for EnvironmentColorPurple {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorRed;
impl Key for EnvironmentColorRed {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorTeal;
impl Key for EnvironmentColorTeal {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorYellow;
impl Key for EnvironmentColorYellow {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray;
impl Key for EnvironmentColorGray {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray2;
impl Key for EnvironmentColorGray2 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray3;
impl Key for EnvironmentColorGray3 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray4;
impl Key for EnvironmentColorGray4 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray5;
impl Key for EnvironmentColorGray5 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorGray6;
impl Key for EnvironmentColorGray6 {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct EnvironmentColorLabel;
impl Key for EnvironmentColorLabel {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSecondaryLabel;
impl Key for EnvironmentColorSecondaryLabel {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorTertiaryLabel;
impl Key for EnvironmentColorTertiaryLabel {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorQuaternaryLabel;
impl Key for EnvironmentColorQuaternaryLabel {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSystemFill;
impl Key for EnvironmentColorSystemFill {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSecondarySystemFill;
impl Key for EnvironmentColorSecondarySystemFill {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorTertiarySystemFill;
impl Key for EnvironmentColorTertiarySystemFill {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorQuaternarySystemFill;
impl Key for EnvironmentColorQuaternarySystemFill {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorPlaceholderText;
impl Key for EnvironmentColorPlaceholderText {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSystemBackground;
impl Key for EnvironmentColorSystemBackground {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSecondarySystemBackground;
impl Key for EnvironmentColorSecondarySystemBackground {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorTertiarySystemBackground;
impl Key for EnvironmentColorTertiarySystemBackground {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorSeparator;
impl Key for EnvironmentColorSeparator {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorOpaqueSeparator;
impl Key for EnvironmentColorOpaqueSeparator {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorLink;
impl Key for EnvironmentColorLink {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorDarkText;
impl Key for EnvironmentColorDarkText {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorLightText;
impl Key for EnvironmentColorLightText {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct EnvironmentColorAccent;
impl Key for EnvironmentColorAccent {
    type Value = Color;
}

// Material colors theme
#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThick;
impl Key for EnvironmentColorUltraThick {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThick;
impl Key for EnvironmentColorThick {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorRegular;
impl Key for EnvironmentColorRegular {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThin;
impl Key for EnvironmentColorThin {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThin;
impl Key for EnvironmentColorUltraThin {
    type Value = Color;
}

// Material colors theme light
#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThickLight;
impl Key for EnvironmentColorUltraThickLight {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThickLight;
impl Key for EnvironmentColorThickLight {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorRegularLight;
impl Key for EnvironmentColorRegularLight {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThinLight;
impl Key for EnvironmentColorThinLight {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThinLight;
impl Key for EnvironmentColorUltraThinLight {
    type Value = Color;
}

// Material colors theme dark
#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThickDark;
impl Key for EnvironmentColorUltraThickDark {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThickDark;
impl Key for EnvironmentColorThickDark {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorRegularDark;
impl Key for EnvironmentColorRegularDark {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorThinDark;
impl Key for EnvironmentColorThinDark {
    type Value = Color;
}

#[derive(Copy, Clone, Debug)]
struct EnvironmentColorUltraThinDark;
impl Key for EnvironmentColorUltraThinDark {
    type Value = Color;
}