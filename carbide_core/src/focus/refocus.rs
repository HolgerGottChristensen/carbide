#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Refocus {
    FocusRequest,
    FocusNext,
    FocusPrevious,
}
