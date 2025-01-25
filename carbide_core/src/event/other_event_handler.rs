use std::any::{Any, TypeId};
use carbide::environment::{AnyDebug, Key};
use carbide::event::CoreEvent;
use crate::draw::InnerImageContext;
use crate::environment::{EnvironmentStack};
use crate::focus::Focusable;
use crate::state::StateSync;
use crate::text::InnerTextContext;
use crate::widget::{CommonWidget, WidgetSync};

pub trait OtherEventHandler: CommonWidget + WidgetSync + Focusable {
    /// This will get called if there are event that are not covered by the other functions.
    /// This will get delegated to all widgets.
    /// It will never get called with mouse or keyboard events.
    /// TODO: Separate touch events.
    #[allow(unused_variables)]
    fn handle_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {}

    fn process_other_event(&mut self, event: &OtherEvent, ctx: &mut OtherEventContext) {
        //if ctx.env.is_event_current() {
            self.sync(ctx.env_stack);
            self.handle_other_event(event, ctx);
        //}

        self.foreach_child_direct(&mut |child| {
            child.process_other_event(event, ctx);
        });
    }
}

pub struct OtherEventContext<'a, 'b: 'a> {
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env_stack: &'a mut EnvironmentStack<'b>,
}

#[derive(Debug)]
pub enum OtherEvent {
    CoreEvent(CoreEvent),
    Key(TypeId),
    KeyValue(TypeId, Box<dyn AnyDebug>)
}

impl OtherEvent {
    pub fn is<K: ?Sized + 'static>(&self) -> bool {
        let type_id = TypeId::of::<K>();
        match self {
            OtherEvent::CoreEvent(a) => a.type_id() == type_id,
            OtherEvent::Key(k) => *k == type_id,
            OtherEvent::KeyValue(k, _) => *k == type_id,
        }
    }

    pub fn value<K: Key>(&self) -> Option<&K::Value> {
        match self {
            OtherEvent::KeyValue(k, value) => value.downcast_ref(),
            _ => None
        }
    }
}