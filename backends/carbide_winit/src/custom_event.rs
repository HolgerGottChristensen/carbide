use accesskit_winit::Event;
use crate::custom_event::CustomEvent::Accessibility;

#[derive(Debug)]
pub enum CustomEvent {
    Core(carbide_core::event::CoreEvent),
    Accessibility(Event)
}

impl From<Event> for CustomEvent {
    fn from(value: Event) -> Self {
        Accessibility(value)
    }
}