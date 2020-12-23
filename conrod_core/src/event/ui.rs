use crate::event::click::Click;
use crate::event::double_click::DoubleClick;
use crate::event::drag::Drag;
use crate::event::motion::Motion;
use crate::event::press::PressEvent;
use crate::event::release::Release;
use crate::event::scroll::Scroll;
use crate::event::tap::Tap;
use crate::event::text::Text;
use crate::input::{Source, Touch};
use crate::position::Dimensions;
use crate::widget;

/// Represents all events interpreted by the `Ui`.
#[derive(Clone, PartialEq, Debug)]
pub enum UiEvent {
    /// Entered text, along with the widget that was capturing the keyboard at the time.
    Text(Option<widget::Id>, Text),
    /// Some button was pressed, along with the widget that was capturing the device whose button
    /// was pressed.
    Press(Option<widget::Id>, PressEvent),
    /// Some button was released, along with the widget that was capturing the device whose button
    /// was released.
    Release(Option<widget::Id>, Release),
    /// Represents all forms of motion input, alongside with the widget that was capturing the
    /// mouse at the time.
    Motion(Option<widget::Id>, Motion),
    /// Interaction with a touch screen/surface.
    Touch(Option<widget::Id>, Touch),
    /// The window's dimensions were resized.
    WindowResized(Dimensions),
    /// Represents a pointing device being pressed and subsequently released while over the same
    /// location.
    Click(Option<widget::Id>, Click),
    /// Two `Click` events with the same `button` and `xy` occurring within a duration that is less
    /// that the `theme.double_click_threshold`.
    DoubleClick(Option<widget::Id>, DoubleClick),
    /// A user tapped a touch screen/surface.
    Tap(Option<widget::Id>, Tap),
    /// Represents a pointing device button being pressed and a subsequent movement of the mouse.
    Drag(Option<widget::Id>, Drag),
    /// A generic scroll event.
    ///
    /// `Scroll` does not necessarily have to get created by a mouse wheel, it could be generated
    /// from a keypress, or as a response to handling some other event.
    ///
    /// Received `Scroll` events are first applied to all scrollable widgets under the mouse from
    /// top to bottom. The remainder will then be applied to either 1. whatever widget captures the
    /// device from which the scroll was emitted or 2. whatever widget was specified.
    Scroll(Option<widget::Id>, Scroll),
    /// Indicates that the given widget has captured the given user input source.
    WidgetCapturesInputSource(widget::Id, Source),
    /// Indicates that the given widget has released the given user input source.
    WidgetUncapturesInputSource(widget::Id, Source),
}


