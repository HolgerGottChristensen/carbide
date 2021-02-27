use crate::prelude::{GlobalState, State};
use crate::state::environment_state::environment_font_size_state::EnvironmentFontSizeState;
use crate::state::state::U32State;

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