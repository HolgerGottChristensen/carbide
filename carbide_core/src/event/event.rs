use crate::event::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::focus::Focusable;
use crate::prelude::Environment;
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait Event: CommonWidget + StateSync + Focusable {
    /// A function that will be called when a mouse event occurs.
    /// It will only get called on the events where the cursor is inside.
    /// Return true if the event is consumed, and will thus not be delegated to other
    /// widgets.
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment);

    /// A function that will get called when a keyboard event occurs.
    /// This event will be given to all widgets, no matter if they are in focus or not.
    /// This is because the focus will be decided by the widgets themselves.
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment);

    /// This will get called if there are event that are not covered by the other functions.
    /// This will get delegated to all widgets.
    /// It will never get called with mouse or keyboard events.
    /// TODO: Separate touch events.
    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment);

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment);

    fn process_mouse_event_default(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment) {
        if !*consumed {
            self.capture_state(env);
            self.handle_mouse_event(event, consumed, env);
            self.release_state(env);
        }

        for child in self.proxied_children() {
            child.process_mouse_event(event, &consumed, env);
            if *consumed { return (); }
        }
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment);

    fn process_keyboard_event_default(&mut self, event: &KeyboardEvent, env: &mut Environment) {
        self.capture_state(env);
        self.handle_keyboard_event(event, env);
        self.release_state(env);

        for child in self.proxied_children() {
            child.process_keyboard_event(event, env);
        }
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment);

    fn process_other_event_default(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.capture_state(env);
        self.handle_other_event(event, env);
        self.release_state(env);

        for child in self.proxied_children() {
            child.process_other_event(event, env);
        }
    }
}