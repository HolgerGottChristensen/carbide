
#[derive(Eq, PartialEq, Clone, Debug, Copy)]
pub enum Focus {
    Focused,
    FocusRequested,
    FocusReleased,
    Unfocused
}