use fxhash::{FxBuildHasher, FxHashMap};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::option::Option::Some;
use std::rc::Rc;

use crate::cursor::MouseCursor;
use crate::draw::Position;
use crate::event::{EventSink, HasEventSink};
use crate::focus::Refocus;
use crate::widget::{AnyWidget, FilterId, ImageFilter, WidgetId};

pub struct Environment {
    /// A map from String to a widget.
    /// This key should correspond to the targeted overlay_layer
    overlay_map: FxHashMap<&'static str, Rc<RefCell<Option<Box<dyn AnyWidget>>>>>,

    /// This field holds the requests for refocus. If Some we need to check the refocus
    /// reason and apply that focus change after the event is done. This also means that
    /// the focus change is not instant, but updates after each run event.
    pub focus_request: Option<Refocus>,

    /// A map that contains an image filter used for the Filter widget.
    filter_map: FxHashMap<FilterId, ImageFilter>,

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
        let filters = HashMap::with_hasher(FxBuildHasher::default());

        let res = Environment {
            overlay_map: HashMap::with_hasher(FxBuildHasher::default()),
            focus_request: None,
            filter_map: filters,
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

    pub fn with_overlay_layer<R, F: FnOnce(&mut Environment)->R>(&mut self, id: &'static str, layer: Rc<RefCell<Option<Box<dyn AnyWidget>>>>, f: F) -> R {
        let old = self.overlay_map.insert(id, layer);

        let res = f(self);

        if let Some(old) = old {
            self.overlay_map.insert(id, old);
        } else {
            self.overlay_map.remove(id);
        }

        res
    }

    pub fn add_overlay(&mut self, id: &'static str, widget: Box<dyn AnyWidget>) {
        if let Some(layer) = self.overlay_map.get_mut(&id) {
            *layer.borrow_mut() = Some(widget);
        } else {
            println!("Cannot add an overlay without a layer");
        }
    }

    pub fn contains_overlay(&self, id: &'static str, widget_id: WidgetId) -> bool {
        if let Some(layer) = self.overlay_map.get(&id) {
            layer.borrow().as_ref().map_or(false, |m| m.id() == widget_id)
        } else {
            println!("Cannot add an overlay without a layer");
            false
        }
    }

    pub fn remove_overlay(&mut self, id: &'static str, widget_id: WidgetId) {
        if self.contains_overlay(id, widget_id) {
            if let Some(layer) = self.overlay_map.get_mut(&id) {
                *layer.borrow_mut() = None;
            } else {
                println!("Cannot add an overlay without a layer");
            }
        }
    }

    pub fn filters(&self) -> &FxHashMap<FilterId, ImageFilter> {
        &self.filter_map
    }

    pub fn insert_filter(&mut self, filter: ImageFilter) -> FilterId {
        let filter_id = FilterId::next();
        self.filter_map.insert(filter_id, filter);
        filter_id
    }

    pub fn remove_filter(&mut self, id: FilterId) {
        self.filter_map.remove(&id);
    }
}

impl HasEventSink for Environment {
    fn event_sink(&self) -> Box<dyn EventSink> {
        self.event_sink.clone()
    }
}