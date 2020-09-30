use Point;
use event::mouse_press::MousePress;
use event::button::ButtonEvent;
use event::key_press::KeyPress;

/// Contains all relevant information for a Press event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PressEvent {
    /// The `Button` that was pressed.
    pub button: ButtonEvent,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

impl PressEvent {

    /// Returns a copy of the Press relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> PressEvent {
        PressEvent {
            button: self.button.relative_to(xy),
            ..*self
        }
    }

    /// If the `Press` event represents the pressing of a mouse button, return `Some`.
    pub fn mouse(self) -> Option<MousePress> {
        match self.button {
            ButtonEvent::Mouse(button, xy) =>
                Some(MousePress {
                    button: button,
                    xy: xy,
                    modifiers: self.modifiers,
                }),
            _ => None,
        }
    }

    /// If the `Press` event represents the pressing of keyboard button, return `Some`.
    pub fn key(self) -> Option<KeyPress> {
        match self.button {
            ButtonEvent::Keyboard(key) =>
                Some(KeyPress {
                    key: key,
                    modifiers: self.modifiers,
                }),
            _ => None,
        }
    }

}



