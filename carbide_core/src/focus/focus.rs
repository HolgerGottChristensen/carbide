#[derive(Eq, PartialEq, Clone, Debug)]
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
