use std::fmt::Formatter;

use crate::draw::Position;
use crate::event::{EventSink, HasEventSink};

pub struct Environment {
    mouse_position: Position,

    event_sink: Box<dyn EventSink>,
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Environment {
    pub fn new(
        event_sink: Box<dyn EventSink>,
    ) -> Self {
        let res = Environment {
            mouse_position: Default::default(),
            event_sink,
        };

        res
    }

    pub fn mouse_position(&self) -> Position {
        self.mouse_position
    }

    pub fn set_mouse_position(&mut self, position: Position) {
        self.mouse_position = position;
    }
}

impl HasEventSink for Environment {
    fn event_sink(&self) -> Box<dyn EventSink> {
        self.event_sink.clone()
    }
}