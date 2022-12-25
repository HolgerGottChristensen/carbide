use crate::environment::EnvironmentFontSizeState;
use crate::state::*;
use crate::state::WidgetState;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum EnvironmentFontSize {
    LargeTitle,
    Title,
    Title2,
    Title3,
    Headline,
    Body,
    Callout,
    Subhead,
    Footnote,
    Caption,
    Caption2,
    Custom(String),
}

impl Default for EnvironmentFontSize {
    fn default() -> Self {
        EnvironmentFontSize::Body
    }
}

impl Into<TState<u32>> for EnvironmentFontSize {
    fn into(self) -> TState<u32> {
        WidgetState::new(Box::new(EnvironmentFontSizeState::new(self)))
    }
}
