use accesskit_winit::Event;

#[derive(Debug)]
pub enum CustomEvent {
    Core(carbide_core::event::CoreEvent),
    Accessibility(Event),
}

impl From<Event> for CustomEvent {
    fn from(value: Event) -> Self {
        CustomEvent::Accessibility(value)
    }
}