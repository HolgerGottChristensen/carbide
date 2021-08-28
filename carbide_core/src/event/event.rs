use crate::event::KeyboardEventHandler;
use crate::event::MouseEventHandler;
use crate::event::OtherEventHandler;

pub trait Event: MouseEventHandler + KeyboardEventHandler + OtherEventHandler {}

impl<T> Event for T where T: MouseEventHandler + KeyboardEventHandler + OtherEventHandler {}
