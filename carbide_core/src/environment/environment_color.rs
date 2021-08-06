use crate::prelude::EnvironmentColorState;
use crate::state::*;
use crate::state::widget_state::WidgetState;
use crate::widget::GlobalStateContract;

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
    Custom(String),
}

impl Default for EnvironmentColor {
    fn default() -> Self {
        EnvironmentColor::Blue
    }
}

impl<GS: GlobalStateContract> Into<ColorState<GS>> for EnvironmentColor {
    fn into(self) -> ColorState<GS> {
        WidgetState::new(Box::new(EnvironmentColorState::new(self)))
    }
}