use crate::widget::GlobalState;
use crate::prelude::State;
use crate::Color;
use crate::state::environment_state::environment_color_state::EnvironmentColorState;
use crate::state::state::ColorState;

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
    Custom(String)
}

impl Default for EnvironmentColor {
    fn default() -> Self {
        EnvironmentColor::Blue
    }
}

impl<GS: GlobalState> Into<ColorState<GS>> for EnvironmentColor {
    fn into(self) -> ColorState<GS> {
        Box::new(EnvironmentColorState::new(self))
    }
}