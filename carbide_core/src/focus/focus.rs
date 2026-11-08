#[derive(Eq, PartialEq, Clone, Debug)]
#[derive(Default)]
pub enum Focus {
    Focused,
    FocusRequested,
    FocusReleased,
    #[default]
    Unfocused,
}

