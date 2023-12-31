use carbide::event::{CustomEvent, MouseEvent, WindowEvent};
use crate::event::KeyboardEvent;

#[derive(Clone, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Window(WindowEvent),
    Custom(CustomEvent),
}

pub trait IntoEvent {
    fn into(self) -> Option<Event>;
}