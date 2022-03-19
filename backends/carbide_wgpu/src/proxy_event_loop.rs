use winit::event_loop::EventLoopProxy;
use carbide_core::event::{CustomEvent, EventSink};

#[derive(Clone)]
pub(crate) struct ProxyEventLoop(pub EventLoopProxy<CustomEvent>);

impl EventSink for ProxyEventLoop {
    fn call(&self, event: CustomEvent) {
        self.0.send_event(event).unwrap();
    }
}