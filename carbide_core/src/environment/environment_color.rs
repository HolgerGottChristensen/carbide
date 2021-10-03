use crate::prelude::EnvironmentColorState;
use crate::state::*;
use crate::state::WidgetState;

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

impl Default for EnvironmentColor {
    fn default() -> Self {
        EnvironmentColor::Blue
    }
}

impl Into<StateKey> for EnvironmentColor {
    fn into(self) -> StateKey {
        StateKey::Color(self)
    }
}

impl Into<ColorState> for EnvironmentColor {
    fn into(self) -> ColorState {
        WidgetState::new(Box::new(EnvironmentColorState::new(self)))
    }
}
