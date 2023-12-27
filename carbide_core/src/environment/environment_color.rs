use crate::draw::Color;
use crate::environment::Environment;
use crate::render::Style;
use crate::state::*;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
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

    Custom(String),
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
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&Environment, &EnvironmentColor)->Color, EnvironmentColor, Color, G>;

    fn convert<F: AnyReadState<T=EnvironmentColor> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| {
            env.get_color(&EnvironmentStateKey::Color(value.clone())).unwrap()
        })
    }
}

impl ConvertIntoRead<Style> for EnvironmentColor {
    type Output<G: AnyReadState<T=Self> + Clone> = EnvMap1<fn(&Environment, &EnvironmentColor)->Style, EnvironmentColor, Style, G>;

    fn convert<F: AnyReadState<T=EnvironmentColor> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map_env(f, |env, value| {
            Style::Color(env.get_color(&EnvironmentStateKey::Color(value.clone())).unwrap())
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