use crate::prelude::GlobalState;
use crate::state::*;
use crate::state::environment_state::environment_font_size_state::EnvironmentFontSizeState;

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
    Custom(String)
}

impl Default for EnvironmentFontSize {
    fn default() -> Self {
        EnvironmentFontSize::Body
    }
}

impl<GS: GlobalState> Into<U32State<GS>> for EnvironmentFontSize {
    fn into(self) -> U32State<GS> {
        Box::new(EnvironmentFontSizeState::new(self))
    }
}