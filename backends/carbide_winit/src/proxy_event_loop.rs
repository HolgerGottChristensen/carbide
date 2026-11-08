use crate::event_loop::EventLoopProxy;

use carbide_core::event::EventSink;
use crate::custom_event::CustomEvent;

#[derive(Clone)]
pub(crate) struct ProxyEventLoop(pub EventLoopProxy<CustomEvent>);

impl EventSink for ProxyEventLoop {
    fn send(&self, event: carbide_core::event::CoreEvent) {
        self.0.send_event(CustomEvent::Core(event)).unwrap();
    }
}
