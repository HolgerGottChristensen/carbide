/// Contains all relevant information for a Text event.
#[derive(Clone, PartialEq, Debug)]
pub struct Text {
    /// All text that was entered as a part of the event.
    pub string: String,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

