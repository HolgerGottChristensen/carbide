use std::collections::HashMap;
use std::time::Instant;
use carbide::event::Event;
use carbide::event::window_event_handler::WindowEventHandler;
use crate::event::{IntoEvent, KeyboardEventHandler, ModifierKey, MouseButton, MouseEvent};
use crate::event::MouseEventHandler;
use crate::event::OtherEventHandler;
use crate::draw::{InnerImageContext, Position};
use crate::environment::Environment;
use crate::text::InnerTextContext;
use crate::widget::AnyWidget;

pub trait EventHandler: MouseEventHandler + KeyboardEventHandler + WindowEventHandler + OtherEventHandler {}

impl<T> EventHandler for T where T: MouseEventHandler + KeyboardEventHandler + WindowEventHandler + OtherEventHandler {}