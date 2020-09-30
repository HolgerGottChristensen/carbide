/// Contains all relevant information for the event where a keyboard button was release.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct KeyRelease {
    /// The key that was release.
    pub key: input::Key,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

