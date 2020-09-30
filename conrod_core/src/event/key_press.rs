/// Contains all relevant information for the event where a keyboard button was pressed.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct KeyPress {
    /// The key that was pressed.
    pub key: input::Key,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

