use Point;

/// Contains all relevant information for the event where a mouse button was released.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MouseRelease {
    /// The mouse button that was released.
    pub button: input::MouseButton,
    /// The location at which the mouse was released.
    pub xy: Point,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

