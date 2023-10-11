use carbide_core::event::{CustomEvent, EventSink};
use winit::event_loop::EventLoopProxy;

#[derive(Clone)]
pub(crate) struct ProxyEventLoop(pub EventLoopProxy<CustomEvent>);

impl EventSink for ProxyEventLoop {
    fn send(&self, event: CustomEvent) {
        self.0.send_event(event).unwrap();
    }
}
