use Point;
use utils::vec2_sub;

/// Contains all the relevant information for a mouse drag.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Drag {
    /// Which mouse button was being held during the drag
    pub button: input::MouseButton,
    /// The point from which the current series of drag events began.
    ///
    /// This will be the position of the pointing device whenever the dragging press began.
    pub origin: Point,
    /// The point from which this drag event began.
    pub from: Point,
    /// The point at which this drag event ended.
    pub to: Point,
    /// The magnitude of the vector between `from` and `to`.
    pub delta_xy: Point,
    /// The magnitude of the vector between `origin` and `to`.
    pub total_delta_xy: Point,
    /// Which modifier keys are being held during the mouse drag.
    pub modifiers: input::keyboard::ModifierKey,
}

impl Drag {
    /// Returns a copy of the Drag relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Drag {
        Drag{
            origin: vec2_sub(self.origin, xy),
            from: vec2_sub(self.from, xy),
            to: vec2_sub(self.to, xy),
            ..*self
        }
    }
}


