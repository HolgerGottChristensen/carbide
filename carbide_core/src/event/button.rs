use serde::{Deserialize, Serialize};

use crate::event::{Key, MouseButton};

#[derive(Copy, Clone, Deserialize, Serialize, PartialEq, PartialOrd, Ord, Eq, Hash, Debug)]
pub enum Button {
    /// A keyboard button.
    Keyboard(Key),
    /// A mouse button.
    Mouse(MouseButton),
    // /// A controller button.
    // Controller(ControllerButton),
    // /// A controller hat (d-Pad)
    // Hat(ControllerHat),
}

impl From<Key> for Button {
    fn from(key: Key) -> Self {
        Button::Keyboard(key)
    }
}

impl From<MouseButton> for Button {
    fn from(btn: MouseButton) -> Self {
        Button::Mouse(btn)
    }
}

// impl From<ControllerButton> for Button {
//     fn from(btn: ControllerButton) -> Self {
//         Button::Controller(btn)
//     }
// }
