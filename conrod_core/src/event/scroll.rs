/// Holds all the relevant information about a scroll event
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Scroll {
    /// The amount of scroll along the x axis.
    pub x: f64,
    /// The amount of scroll along the y axis.
    pub y: f64,
    /// Which modifier keys, if any, that were being held down while the scroll occured
    pub modifiers: input::keyboard::ModifierKey,
}

