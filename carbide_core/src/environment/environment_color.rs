use crate::Color;
use crate::color::WHITE;
use crate::environment::EnvironmentColorState;
use crate::render::Style;
use crate::state::*;
use crate::state::WidgetState;
use crate::widget::AdvancedColor;

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

/*impl Into<StateKey> for EnvironmentColor {
    fn into(self) -> StateKey {
        StateKey::Color(self)
    }
}

impl Into<TState<Color>> for EnvironmentColor {
    fn into(self) -> TState<Color> {
        self.state()
    }
}*/

/*impl Into<TState<Color>> for TState<EnvironmentColor> {
    fn into(self) -> TState<Color> {
        let state = Map1::read_map(self, |e: &EnvironmentColor| e.state()).ignore_writes();

        Flatten::new(state.as_dyn())
    }
}*/

/*impl Into<TState<Style>> for EnvironmentColor {
    fn into(self) -> TState<Style> {
        let state: TState<Color> = WidgetState::new(Box::new(EnvironmentColorState::new(self)));
        let state: RState<Style> = state.into();
        state.ignore_writes()
    }
}*/

/*impl Into<TState<Style>> for TState<EnvironmentColor> {
    fn into(self) -> TState<Style> {
        let state: TState<Color> = self.into();

        Map1::read_map(state, |s: &Color| Style::Color(*s)).ignore_writes()
    }
}*/

/*impl<T> IntoReadState<Style> for T where T: AnyReadState<T=Color> + Clone {
    type Output = RMap1<fn(&Color)->Style, Color, Style, T>;

    fn into_read_state(self) -> Self::Output {
        Map1::read_map(self, |c| {
            Style::Color(*c)
        })
    }
}*/

/*impl IntoReadState<Style> for EnvironmentColor {
    type Output = EnvironmentColorState;

    fn into_read_state(self) -> Self::Output {
        EnvironmentColorState::new(self)
    }
}
*/


/*impl IntoReadState<Color> for EnvironmentColor {
    type Output = RMap1<fn(&Style) -> Color, Style, Color, EnvironmentColorState>;

    fn into_read_state(self) -> Self::Output {
        /*Map1::read_map(EnvironmentColorState::new(self), |s| {
            match s {
                Style::Color(c) => *c,
                _ => WHITE,
            }
        })*/
        todo!()
    }
}*/