use event::press::PressEvent;
use event::click::Click;
use event::drag::Drag;
use event::motion::Motion;
use event::release::Release;
use event::scroll::Scroll;
use event::tap::Tap;
use event::text::Text;
use event::double_click::DoubleClick;
use position::Dimensions;
use input;

impl From<PressEvent> for WidgetEvent {
    fn from(press: PressEvent) -> Self {
        WidgetEvent::Press(press)
    }
}

impl From<Click> for WidgetEvent {
    fn from(click: Click) -> Self {
        WidgetEvent::Click(click)
    }
}

impl From<Drag> for WidgetEvent {
    fn from(drag: Drag) -> Self {
        WidgetEvent::Drag(drag)
    }
}

impl From<input::Touch> for WidgetEvent {
    fn from(touch: input::Touch) -> Self {
        WidgetEvent::Touch(touch)
    }
}

impl From<Motion> for WidgetEvent {
    fn from(motion: Motion) -> Self {
        WidgetEvent::Motion(motion)
    }
}

impl From<Release> for WidgetEvent {
    fn from(release: Release) -> Self {
        WidgetEvent::Release(release)
    }
}

impl From<Scroll> for WidgetEvent {
    fn from(scroll: Scroll) -> Self {
        WidgetEvent::Scroll(scroll)
    }
}

impl From<Tap> for WidgetEvent {
    fn from(tap: Tap) -> Self {
        WidgetEvent::Tap(tap)
    }
}

impl From<Text> for WidgetEvent {
    fn from(text: Text) -> Self {
        WidgetEvent::Text(text)
    }
}

/// Events that apply to a specific widget.
///
/// Rather than delivering entire `event::Event`s to the widget (with a lot of redundant
/// information), this `event::Widget` is used as a refined, widget-specific event.
///
/// All `Widget` event co-ordinates will be relative to the centre of the `Widget` to which they
/// are delivered.
#[derive(Clone, PartialEq, Debug)]
pub enum WidgetEvent {
    /// Entered text.
    Text(Text),
    /// Represents all forms of motion input.
    Motion(Motion),
    /// Interaction with a touch screen.
    Touch(input::Touch),
    /// Some button was pressed.
    Press(PressEvent),
    /// Some button was released.
    Release(Release),
    /// Represents a pointing device being pressed and subsequently released while over the same
    /// location.
    Click(Click),
    /// Two `Click` events with the same `button` and `xy` occurring within a duration that is less
    /// that the `theme.double_click_threshold`.
    DoubleClick(DoubleClick),
    /// A user tapped the widget on a touch screen/surface.
    Tap(Tap),
    /// Represents a pointing device button being pressed and a subsequent movement of the mouse.
    Drag(Drag),
    /// Represents the amount of scroll that has been applied to this widget.
    Scroll(Scroll),
    /// The window's dimensions were resized.
    WindowResized(Dimensions),
    /// The widget has captured the given input source.
    CapturesInputSource(input::Source),
    /// The widget has released the input source from capturing.
    UncapturesInputSource(input::Source),
}

impl From<DoubleClick> for WidgetEvent {
    fn from(double_click: DoubleClick) -> Self {
        WidgetEvent::DoubleClick(double_click)
    }
}

