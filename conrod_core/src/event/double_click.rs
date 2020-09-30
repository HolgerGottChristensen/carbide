use utils::vec2_sub;
use Point;

/// Contains all the relevant information for a double click.
///
/// When handling this event, be sure to check that you are handling the intended `button` too.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DoubleClick {
    /// Which mouse button was clicked
    pub button: input::MouseButton,
    /// The position at which the mouse was released.
    pub xy: Point,
    /// Which modifier keys, if any, that were being held down when the user clicked
    pub modifiers: input::keyboard::ModifierKey,
}

impl DoubleClick {
    /// Returns a copy of the DoubleClick relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> DoubleClick {
        DoubleClick {
            xy: vec2_sub(self.xy, xy),
            ..*self
        }
    }
}

