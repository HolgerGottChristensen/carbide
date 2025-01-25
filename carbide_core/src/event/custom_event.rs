use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use dyn_clone::DynClone;
use crate::environment::EnvironmentKey;

#[derive(Debug, Clone, Copy)]
pub enum CoreEvent {
    Async,
    AsyncStream,
}

pub trait EventSink: DynClone + Send {
    fn send(&self, _: CoreEvent);
}

dyn_clone::clone_trait_object!(EventSink);

impl EnvironmentKey for dyn EventSink {
    type Value = Arc<dyn EventSink>;
}

impl Debug for dyn EventSink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("dyn EventSink")
    }
}

#[derive(Clone)]
pub struct NoopEventSink;

impl EventSink for NoopEventSink {
    fn send(&self, _: CoreEvent) {}
}


pub trait HasEventSink {
    fn event_sink(&self) -> Box<dyn EventSink>;
}