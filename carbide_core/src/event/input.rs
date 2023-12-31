use crate::event::{Button, CustomEvent, ModifierKey};
use crate::event::keyboard_event_handler::Ime;
use crate::event::Motion;
use crate::event::types::gesture::Gesture;
use crate::event::types::touch::Touch;

/// The event type that is used by carbide to track inputs from the world. Events yielded by polling
/// window backends should be converted to this type. This can be thought of as the event type
/// which is supplied by the window backend to drive the state of the `Ui` forward.
///
/// This type is solely used within the `Ui::handle_event` method.  The `Input` events are
/// interpreted to create higher level `Event`s (such as DoubleClick, WidgetCapturesKeyboard, etc)
/// which are stored for later processing by `Widget`s, which will occur during the call to
/// `Ui::set_widgets`.
///
/// **Note:** `Input` events that contain co-ordinates must be oriented with (0, 0) at the middle
/// of the window with the *y* axis pointing upwards (Cartesian co-ordinates). All co-ordinates and
/// dimensions must be given as `Scalar` (DPI agnostic) values. Many windows provide coordinates
/// with the origin in the top left with *y* pointing down, so you might need to translate these
/// co-ordinates when converting to this event. Also be sure to invert the *y* axis of MouseScroll
/// events.
#[derive(Clone, Debug)]
pub enum Input {
    /// A button on some input device was pressed.
    Press(Button),
    /// A button on some input device was released.
    Release(Button),
    /// The window was received to the given dimensions.
    Resize(f64, f64),
    /// Some motion input was received (e.g. moving mouse or joystick axis).
    Motion(Motion),
    /// Gesture events like rotate, scale and smart scale (two finger double click)
    Gesture(Gesture),
    /// Input from a touch surface/screen.
    Touch(Touch),
    /// The window was focused or lost focus.
    Focus(bool),
    /// The backed requested to redraw.
    Redraw,
    CloseRequested,
    /// Custom carbide event
    Custom(CustomEvent),
    ModifiersChanged(ModifierKey),
    Ime(Ime),
}

impl From<Touch> for Input {
    fn from(touch: Touch) -> Self {
        Input::Touch(touch)
    }
}

impl From<Motion> for Input {
    fn from(motion: Motion) -> Self {
        Input::Motion(motion)
    }
}
