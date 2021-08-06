use crate::prelude::EnvironmentFontSizeState;
use crate::prelude::GlobalStateContract;
use crate::state::*;
use crate::state::widget_state::WidgetState;

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

impl<GS: GlobalStateContract> Into<U32State<GS>> for EnvironmentFontSize {
    fn into(self) -> U32State<GS> {
        WidgetState::new(Box::new(EnvironmentFontSizeState::new(self)))
    }
}