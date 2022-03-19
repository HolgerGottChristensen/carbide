use crate::environment::Environment;
use crate::event::KeyboardEvent;
use crate::focus::Focusable;
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait KeyboardEventHandler: CommonWidget + StateSync + Focusable {
    /// A function that will get called when a keyboard event occurs.
    /// This event will be given to all widgets, no matter if they are in focus or not.
    /// This is because the focus will be decided by the widgets themselves.
    #[allow(unused_variables)]
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {}

    /// This function is used to delegate the keyboard events, first to its own handle event
    /// [KeyboardEventHandler::handle_keyboard_event()] and then to the direct children.
    /// Notice this means that proxy widgets will receive the events and should make sure to
    /// delegate events to their children themselves. This is opposed to layout where the
    /// proxy widgets will be skipped in the tree. If you override this, you will need to
    /// manage the events yourself. Overriding this you are therby able to restrict events to
    /// a widgets children.
    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.capture_state(env);
        self.handle_keyboard_event(event, env);
        self.release_state(env);

        for mut child in self.children_direct() {
            child.process_keyboard_event(event, env);
        }
    }
}
