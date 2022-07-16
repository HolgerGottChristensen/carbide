use dyn_clone::DynClone;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CustomEvent {
    Async,
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
