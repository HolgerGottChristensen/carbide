use crate::event::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::event::KeyboardEventHandler;
use crate::event::MouseEventHandler;
use crate::event::OtherEventHandler;
use crate::focus::Focusable;
use crate::prelude::Environment;
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait Event: MouseEventHandler + KeyboardEventHandler + OtherEventHandler {}

impl<T> Event for T where T: MouseEventHandler + KeyboardEventHandler + OtherEventHandler {}