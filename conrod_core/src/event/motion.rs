use Point;

/// Contains all relevant information for a Motion event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Motion {
    /// The type of `Motion` that occurred.
    pub motion: input::Motion,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

impl Motion {
    /// Returns a copy of the `Motion` relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Motion {
        let motion = match self.motion {
            input::Motion::MouseCursor { x, y } =>
                input::Motion::MouseCursor { x: x - xy[0], y: y - xy[1] },
            motion => motion,
        };
        Motion {
            motion: motion,
            ..*self
        }
    }
}

