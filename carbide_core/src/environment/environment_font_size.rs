use crate::prelude::EnvironmentFontSizeState;
use crate::state::WidgetState;
use crate::state::*;

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

impl Into<U32State> for EnvironmentFontSize {
    fn into(self) -> U32State {
        WidgetState::new(Box::new(EnvironmentFontSizeState::new(self)))
    }
}
