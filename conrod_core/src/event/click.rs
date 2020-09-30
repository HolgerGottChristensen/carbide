use Point;
use utils::vec2_sub;

/// Contains all the relevant information for a mouse click.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Click {
    /// Which mouse button was clicked
    pub button: input::MouseButton,
    /// The position at which the mouse was released.
    pub xy: Point,
    /// Which modifier keys, if any, that were being held down when the user clicked
    pub modifiers: input::keyboard::ModifierKey,
}

impl Click {
    /// Returns a copy of the Click relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Click {
        Click {
            xy: vec2_sub(self.xy, xy),
            ..*self
        }
    }
}

