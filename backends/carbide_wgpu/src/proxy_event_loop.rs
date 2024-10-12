use carbide_winit::event_loop::EventLoopProxy;

use carbide_core::event::{EventSink};
use carbide_winit::custom_event::CustomEvent;

#[derive(Clone)]
pub(crate) struct ProxyEventLoop(pub EventLoopProxy<CustomEvent>);

impl EventSink for ProxyEventLoop {
    fn send(&self, event: carbide_core::event::CoreEvent) {
        self.0.send_event(CustomEvent::Core(event)).unwrap();
    }
}
