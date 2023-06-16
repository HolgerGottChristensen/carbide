use crate::Color;
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
    pub fn color(&self) -> impl ReadState<T=Color> {
        <EnvironmentColor as IntoReadState<Color>>::into_read_state(self.clone())
    }

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

impl<T> IntoReadStateHelper<T, EnvironmentColor, Color> for T where T: AnyReadState<T=EnvironmentColor> + Clone {
    type Output = EnvMap1<fn(&Environment, &EnvironmentColor)->Color, EnvironmentColor, Color, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map_env(self, |env, value| {
            env.get_color(&StateKey::Color(value.clone())).unwrap()
        })
    }
}

impl<T> IntoReadStateHelper<T, EnvironmentColor, Style> for T where T: AnyReadState<T=EnvironmentColor> + Clone {
    type Output = EnvMap1<fn(&Environment, &EnvironmentColor)->Style, EnvironmentColor, Style, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map_env(self, |env, value| {
            Style::Color(env.get_color(&StateKey::Color(value.clone())).unwrap())
        })
    }
}