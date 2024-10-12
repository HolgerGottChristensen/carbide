use crate::event::accessibility_event_handler::AccessibilityEventHandler;
use crate::event::KeyboardEventHandler;
use crate::event::MouseEventHandler;
use crate::event::OtherEventHandler;
use crate::event::WindowEventHandler;

pub trait EventHandler: MouseEventHandler + KeyboardEventHandler + WindowEventHandler + AccessibilityEventHandler + OtherEventHandler {}

impl<T> EventHandler for T where T: MouseEventHandler + KeyboardEventHandler + WindowEventHandler + AccessibilityEventHandler + OtherEventHandler {}