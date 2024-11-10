use std::fmt::Formatter;

use crate::cursor::MouseCursor;
use crate::draw::Position;
use crate::event::{EventSink, HasEventSink};

pub struct Environment {
    cursor: MouseCursor,
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
            cursor: MouseCursor::Default,
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
    
    pub fn cursor(&self) -> MouseCursor {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: MouseCursor) {
        self.cursor = cursor;
    }
}

impl HasEventSink for Environment {
    fn event_sink(&self) -> Box<dyn EventSink> {
        self.event_sink.clone()
    }
}