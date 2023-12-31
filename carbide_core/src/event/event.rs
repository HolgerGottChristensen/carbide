use carbide::event::{CustomEvent, MouseEvent, TouchEvent, WindowEvent};
use crate::event::KeyboardEvent;

#[derive(Clone, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Window(WindowEvent),
    Touch(TouchEvent),
    Custom(CustomEvent),
    DoneProcessingEvents,
}

pub trait IntoEvent {
    fn into(self) -> Option<Event>;
}