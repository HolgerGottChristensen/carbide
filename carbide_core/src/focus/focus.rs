use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Focus {
    Focused,
    FocusRequested,
    FocusReleased,
    Unfocused,
}

impl Default for Focus {
    fn default() -> Self {
        Focus::Unfocused
    }
}
