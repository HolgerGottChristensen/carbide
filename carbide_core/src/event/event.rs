use carbide::event::{CoreEvent, MouseEvent, WindowEvent};
use crate::event::KeyboardEvent;

#[derive(Clone, Debug)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
    Window(WindowEvent),
    CoreEvent(CoreEvent),
}

pub trait IntoEvent {
    fn into(self) -> Option<Event>;
}