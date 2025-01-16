use accesskit_winit::Event;
use std::any::TypeId;

#[derive(Debug)]
pub enum CustomEvent {
    Core(carbide_core::event::CoreEvent),
    Accessibility(Event),
    Key(TypeId),
}

impl From<Event> for CustomEvent {
    fn from(value: Event) -> Self {
        CustomEvent::Accessibility(value)
    }
}