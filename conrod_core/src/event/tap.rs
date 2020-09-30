use Point;
use utils::vec2_sub;
use input;

/// All relevant information for a touch-screen tap event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Tap {
    /// The unique identifier of the source of the touch.
    pub id: input::touch::Id,
    /// The position at which the finger left the screen.
    pub xy: Point,
}

impl Tap {
    /// Returns a copy of the `Tap` relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Self {
        Tap {
            xy: vec2_sub(self.xy, xy),
            ..*self
        }
    }
}

