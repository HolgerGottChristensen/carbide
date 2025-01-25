use std::fmt::Formatter;

use crate::draw::Position;
use crate::event::{EventSink, HasEventSink};

pub struct Environment {

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
            event_sink,
        };

        res
    }
}

impl HasEventSink for Environment {
    fn event_sink(&self) -> Box<dyn EventSink> {
        self.event_sink.clone()
    }
}