use crate::draw::Color;
use crate::environment::{Environment, EnvironmentStack, Key, Keyable};
use crate::render::Style;
use crate::state::*;
use crate::widget::EnvKey;

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

    Custom(&'static str),
}

impl Keyable for EnvironmentColor {
    type Output = Color;

    fn get(&self, stack: &EnvironmentStack) -> Self::Output {
        match self {
            EnvironmentColor::Accent => stack.get::<EnvironmentColorAccent>().cloned().unwrap(),
            _ => todo!()
        }
    }
}

impl EnvKey for EnvironmentColor {
    fn key(&self) -> &'static str {
        match self {
            EnvironmentColor::Blue => "EnvironmentColor::Blue",
            EnvironmentColor::Green => "EnvironmentColor::Green",
            EnvironmentColor::Indigo => "EnvironmentColor::Indigo",
            EnvironmentColor::Orange => "EnvironmentColor::Orange",
            EnvironmentColor::Pink => "EnvironmentColor::Pink",
            EnvironmentColor::Purple => "EnvironmentColor::Purple",
            EnvironmentColor::Red => "EnvironmentColor::Red",
            EnvironmentColor::Teal => "EnvironmentColor::Teal",
            EnvironmentColor::Yellow => "EnvironmentColor::Yellow",
            EnvironmentColor::Gray => "EnvironmentColor::Gray",
            EnvironmentColor::Gray2 => "EnvironmentColor::Gray2",
            EnvironmentColor::Gray3 => "EnvironmentColor::Gray3",
            EnvironmentColor::Gray4 => "EnvironmentColor::Gray4",
            EnvironmentColor::Gray5 => "EnvironmentColor::Gray5",
            EnvironmentColor::Gray6 => "EnvironmentColor::Gray6",
            EnvironmentColor::Label => "EnvironmentColor::Label",
            EnvironmentColor::SecondaryLabel => "EnvironmentColor::SecondaryLabel",
            EnvironmentColor::TertiaryLabel => "EnvironmentColor::TertiaryLabel",
            EnvironmentColor::QuaternaryLabel => "EnvironmentColor::QuaternaryLabel",
            EnvironmentColor::SystemFill => "EnvironmentColor::SystemFill",
            EnvironmentColor::SecondarySystemFill => "EnvironmentColor::SecondarySystemFill",
            EnvironmentColor::TertiarySystemFill => "EnvironmentColor::TertiarySystemFill",
            EnvironmentColor::QuaternarySystemFill => "EnvironmentColor::QuaternarySystemFill",
            EnvironmentColor::PlaceholderText => "EnvironmentColor::PlaceholderText",
            EnvironmentColor::SystemBackground => "EnvironmentColor::SystemBackground",
            EnvironmentColor::SecondarySystemBackground => "EnvironmentColor::SecondarySystemBackground",
            EnvironmentColor::TertiarySystemBackground => "EnvironmentColor::TertiarySystemBackground",
            EnvironmentColor::Separator => "EnvironmentColor::Separator",
            EnvironmentColor::OpaqueSeparator => "EnvironmentColor::OpaqueSeparator",
            EnvironmentColor::Link => "EnvironmentColor::Link",
            EnvironmentColor::DarkText => "EnvironmentColor::DarkText",
            EnvironmentColor::LightText => "EnvironmentColor::LightText",
            EnvironmentColor::Accent => "EnvironmentColor::Accent",
            EnvironmentColor::UltraThick => "EnvironmentColor::UltraThick",
            EnvironmentColor::Thick => "EnvironmentColor::Thick",
            EnvironmentColor::Regular => "EnvironmentColor::Regular",
            EnvironmentColor::Thin => "EnvironmentColor::Thin",
            EnvironmentColor::UltraThin => "EnvironmentColor::UltraThin",
            EnvironmentColor::UltraThickLight => "EnvironmentColor::UltraThickLight",
            EnvironmentColor::ThickLight => "EnvironmentColor::ThickLight",
            EnvironmentColor::RegularLight => "EnvironmentColor::RegularLight",
            EnvironmentColor::ThinLight => "EnvironmentColor::ThinLight",
            EnvironmentColor::UltraThinLight => "EnvironmentColor::UltraThinLight",
            EnvironmentColor::UltraThickDark => "EnvironmentColor::UltraThickDark",
            EnvironmentColor::ThickDark => "EnvironmentColor::ThickDark",
            EnvironmentColor::RegularDark => "EnvironmentColor::RegularDark",
            EnvironmentColor::ThinDark => "EnvironmentColor::ThinDark",
            EnvironmentColor::UltraThinDark => "EnvironmentColor::UltraThinDark",
            EnvironmentColor::Custom(c) => *c,
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
            //env.color(*value).unwrap()
            todo!()
        })
    }
}

impl ConvertIntoRead<Style> for EnvironmentColor {
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&EnvironmentStack, &EnvironmentColor)->Style, EnvironmentColor, Style, G>;

    fn convert<F: AnyReadState<T=EnvironmentColor> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| {
            //Style::Color(env.color(*value).unwrap())
            todo!()
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
struct EnvironmentColorAccent;
impl Key for EnvironmentColorAccent {
    type Value = Color;
}