use dyn_clone::DynClone;
use crate::widget::Widget;

#[derive(Debug, Clone)]
pub enum CustomEvent {
    Async,
    AsyncStream,
}

pub trait EventSink: DynClone + Send {
    fn call(&self, _: CustomEvent);
}

dyn_clone::clone_trait_object!(EventSink);

#[derive(Clone)]
pub struct NoopEventSink;

impl EventSink for NoopEventSink {
    fn call(&self, _: CustomEvent) {

    }
}


pub trait HasEventSink {
    fn event_sink(&self) -> Box<dyn EventSink>;
}