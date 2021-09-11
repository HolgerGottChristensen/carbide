use crate::environment::Environment;
use crate::event::KeyboardEvent;
use crate::focus::Focusable;
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait KeyboardEventHandler: CommonWidget + StateSync + Focusable {
    /// A function that will get called when a keyboard event occurs.
    /// This event will be given to all widgets, no matter if they are in focus or not.
    /// This is because the focus will be decided by the widgets themselves.
    fn handle_keyboard_event(&mut self, _event: &KeyboardEvent, _env: &mut Environment) {}

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.capture_state(env);
        self.handle_keyboard_event(event, env);
        self.release_state(env);

        for child in self.children_direct() {
            child.process_keyboard_event(event, env);
        }
    }
}
