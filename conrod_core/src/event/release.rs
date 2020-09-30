use Point;
use event::mouse_release::MouseRelease;
use event::button::ButtonEvent;
use event::key_release::KeyRelease;

/// Contains all relevant information for a Release event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Release {
    /// The `Button` that was released.
    pub button: ButtonEvent,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}


impl Release {

    /// Returns a copy of the Release relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Release {
        Release {
            button: self.button.relative_to(xy),
            ..*self
        }
    }

    /// If the `Release` event represents the releasing of a mouse button, return `Some`.
    pub fn mouse(self) -> Option<MouseRelease> {
        match self.button {
            ButtonEvent::Mouse(button, xy) =>
                Some(MouseRelease {
                    button: button,
                    xy: xy,
                    modifiers: self.modifiers,
                }),
            _ => None,
        }
    }

    /// If the `Release` event represents the release of keyboard button, return `Some`.
    pub fn key(self) -> Option<KeyRelease> {
        match self.button {
            ButtonEvent::Keyboard(key) =>
                Some(KeyRelease {
                    key: key,
                    modifiers: self.modifiers,
                }),
            _ => None,
        }
    }

}


