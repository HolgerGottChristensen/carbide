use crate::Point;

/// Contains all relevant information for the event where a mouse button was pressed.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MousePress {
    /// The mouse button that was pressed.
    pub button: input::MouseButton,
    /// The location at which the mouse was pressed.
    pub xy: Point,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

