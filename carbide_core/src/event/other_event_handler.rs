use crate::draw::ImageContext;
use crate::environment::Environment;
use crate::focus::Focusable;
use crate::misc::any_debug::AnyDebug;
use crate::text::TextContext;
use crate::widget::{CommonWidget, WidgetSync};
use crate::event::CoreEvent;
use std::any::{Any, TypeId};
use std::ops::Deref;

pub trait OtherEventHandler: CommonWidget + WidgetSync + Focusable {
    /// This will get called if there are event that are not covered by the other functions.
    /// This will get delegated to all widgets.
    /// It will never get called with mouse or keyboard events.
    /// TODO: Separate touch events.
    #[allow(unused_variables)]
    fn handle_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {}

    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        //if ctx.env.is_event_current() {
            self.sync(ctx.env);
            self.handle_other_event(event, ctx);
        //}

        self.foreach_child_direct(&mut |child| {
            child.process_other_event(event, ctx);
        });
    }
}

pub struct OtherEventContext<'a, 'b: 'a> {
    pub text: &'a mut dyn TextContext,
    pub image: &'a mut dyn ImageContext,
    pub env: &'a mut Environment<'b>,
    pub is_current: &'a bool,
    pub is_consumed: &'a mut bool
}

#[derive(Debug)]
pub enum OtherEvent {
    CoreEvent(CoreEvent),
    Dynamic(Box<dyn AnyDebug>)
}

impl OtherEvent {
    pub fn new<K: AnyDebug>(value: K) -> Self {
        OtherEvent::Dynamic(Box::new(value))
    }

    pub fn is<K: AnyDebug>(&self) -> bool {
        let type_id = TypeId::of::<K>();
        match self {
            OtherEvent::CoreEvent(a) => a.type_id() == type_id,
            OtherEvent::Dynamic(value) => value.deref().type_id() == type_id,
        }
    }

    pub fn value<K: AnyDebug>(&self) -> Option<&K> {
        let type_id = TypeId::of::<K>();
        match self {
            OtherEvent::Dynamic(value) => {
                if value.deref().type_id() == type_id {
                    value.downcast_ref()
                } else {
                    None
                }
            },
            _ => None
        }
    }
}