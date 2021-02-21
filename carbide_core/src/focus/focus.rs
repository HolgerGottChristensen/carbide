use serde::{Serialize, Deserialize};

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Focus {
    Focused,
    FocusRequested,
    FocusReleased,
    Unfocused
}