use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use dyn_clone::DynClone;
use carbide::environment::Shared;
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

/// The key used to look up the [EventSink] from the environment.
#[derive(Copy, Clone, Debug)]
pub struct EventSinkKey;

impl EnvironmentKey for EventSinkKey {
    type Value = Arc<dyn EventSink>;
    type Access = Shared;
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