pub use custom_event::CoreEvent;
pub use custom_event::EventSink;
pub use custom_event::HasEventSink;
pub use custom_event::HasRawWindowHandleAndEventSink;
pub use custom_event::NoopEventSink;
pub use event_handler::EventHandler;
pub use keyboard_event_handler::*;
pub use mouse_event_handler::*;
pub use other_event_handler::*;
pub use window_event_handler::*;
pub use accessibility_event_handler::*;
pub use types::hot_key::*;
pub use types::key::*;
pub use types::modifier_key::ModifierKey;
pub use types::touch::*;
pub use types::gesture::*;
pub use event::*;

mod custom_event;
mod event_handler;
mod keyboard_event_handler;
mod mouse_event_handler;
mod other_event_handler;
mod types;
mod event;
mod window_event_handler;
mod accessibility_event_handler;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct EventId(u32);

impl EventId {
    pub fn new(id: u32) -> EventId {
        EventId(id)
    }
}