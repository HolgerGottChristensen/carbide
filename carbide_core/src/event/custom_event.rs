use dyn_clone::DynClone;
use raw_window_handle::HasRawWindowHandle;

#[derive(Debug, Clone)]
pub enum CustomEvent {
    Async,
    AsyncStream,
}

pub trait EventSink: DynClone + Send {
    fn send(&self, _: CustomEvent);
}

dyn_clone::clone_trait_object!(EventSink);

#[derive(Clone)]
pub struct NoopEventSink;

impl EventSink for NoopEventSink {
    fn send(&self, _: CustomEvent) {

    }
}


pub trait HasEventSink {
    fn event_sink(&self) -> Box<dyn EventSink>;
}

pub trait HasRawWindowHandleAndEventSink: HasRawWindowHandle + HasEventSink {}

impl<T> HasRawWindowHandleAndEventSink for T where T: HasRawWindowHandle + HasEventSink {}