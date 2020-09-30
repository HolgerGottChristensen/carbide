use Point;
use utils::vec2_sub;

impl ButtonEvent {
    /// Returns a copy of the Button relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> ButtonEvent {
        match *self {
            ButtonEvent::Mouse(m_button, self_xy) => ButtonEvent::Mouse(m_button, vec2_sub(self_xy, xy)),
            button => button,
        }
    }
}

/// The different kinds of `Button`s that may be `Press`ed or `Release`d.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ButtonEvent {
    /// A keyboard button.
    Keyboard(input::Key),
    /// A mouse button along with the location at which it was `Press`ed/`Release`d.
    Mouse(input::MouseButton, Point),
    /// A controller button.
    Controller(input::ControllerButton),
}

