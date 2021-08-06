use crate::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::focus::Focusable;
use crate::prelude::Environment;
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::state::state_sync::StateSync;
use crate::widget::common_widget::CommonWidget;

pub trait Event<GS>: CommonWidget<GS> + StateSync<GS> + Focusable<GS> where GS: GlobalStateContract {
    /// A function that will be called when a mouse event occurs.
    /// It will only get called on the events where the cursor is inside.
    /// Return true if the event is consumed, and will thus not be delegated to other
    /// widgets.
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>);

    /// A function that will get called when a keyboard event occurs.
    /// This event will be given to all widgets, no matter if they are in focus or not.
    /// This is because the focus will be decided by the widgets themselves.
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>);

    /// This will get called if there are event that are not covered by the other functions.
    /// This will get delegated to all widgets.
    /// It will never get called with mouse or keyboard events.
    /// TODO: Separate touch events.
    fn handle_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<GS>);

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>);

    fn process_mouse_event_default(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        if !*consumed {
            self.capture_state(env, global_state);
            self.handle_mouse_event(event, consumed, env);
            self.release_state(env);
        }

        for child in self.get_proxied_children() {
            child.process_mouse_event(event, &consumed, env, global_state);
            if *consumed { return (); }
        }
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>);

    fn process_keyboard_event_default(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.capture_state(env, global_state);
        self.handle_keyboard_event(event, env);
        self.release_state(env);

        for child in self.get_proxied_children() {
            child.process_keyboard_event(event, env, global_state);
        }
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>);

    fn process_other_event_default(&mut self, event: &WidgetEvent, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        self.capture_state(env, global_state);
        self.handle_other_event(event, env);
        self.release_state(env);

        for child in self.get_proxied_children() {
            child.process_other_event(event, env, global_state);
        }
    }
}