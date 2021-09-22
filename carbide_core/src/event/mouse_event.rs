use crate::environment::Environment;
use crate::event::MouseEvent;
use crate::focus::Focusable;
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait MouseEventHandler: CommonWidget + StateSync + Focusable {
    /// A function that will be called when a mouse event occurs.
    /// It will only get called on the events where the cursor is inside.
    /// Return true if the event is consumed, and will thus not be delegated to other
    /// widgets.
    fn handle_mouse_event(&mut self, _event: &MouseEvent, _consumed: &bool, _env: &mut Environment) {}

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        if !*consumed {
            self.capture_state(env);
            self.handle_mouse_event(event, consumed, env);
            self.release_state(env);
        }

        for mut child in self.children_direct() {
            child.process_mouse_event(event, &consumed, env);
            if *consumed {
                return ();
            }
        }
    }
}
