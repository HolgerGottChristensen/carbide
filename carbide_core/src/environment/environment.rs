use fxhash::{FxBuildHasher, FxHashMap};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::option::Option::Some;

use crate::cursor::MouseCursor;
use crate::draw::Position;
use crate::event::{EventSink, HasEventSink};
use crate::focus::Refocus;
use crate::widget::{FilterId, ImageFilter};

pub struct Environment {
    /// This field holds the requests for refocus. If Some we need to check the refocus
    /// reason and apply that focus change after the event is done. This also means that
    /// the focus change is not instant, but updates after each run event.
    pub focus_request: Option<Refocus>,

    cursor: MouseCursor,
    mouse_position: Position,

    event_sink: Box<dyn EventSink>,

    request_application_close: bool,
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
            focus_request: None,
            cursor: MouseCursor::Default,
            mouse_position: Default::default(),
            event_sink,
            request_application_close: false,
        };

        res
    }

    pub fn mouse_position(&self) -> Position {
        self.mouse_position
    }

    pub fn set_mouse_position(&mut self, position: Position) {
        self.mouse_position = position;
    }

    pub fn close_application(&mut self) {
        self.request_application_close = true;
    }

    pub fn should_close_application(&self) -> bool {
        self.request_application_close
    }

    pub fn cursor(&self) -> MouseCursor {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: MouseCursor) {
        self.cursor = cursor;
    }

    /// This method is used to request focus. The focus handling is done after each event is
    /// completed, meaning that this can be called multiple times per frame. The default
    /// behavior is on tab and shift-tab, but you can avoid this by having a widget call
    /// [Self::reset_focus_requests].
    pub fn request_focus(&mut self, request_type: Refocus) {
        self.focus_request = Some(request_type);
    }

    pub fn reset_focus_requests(&mut self) {
        self.focus_request = None;
    }
}

impl HasEventSink for Environment {
    fn event_sink(&self) -> Box<dyn EventSink> {
        self.event_sink.clone()
    }
}