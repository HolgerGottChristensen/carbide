use crate::event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use crate::state::environment::Environment;
use crate::state::state_sync::StateSync;
use crate::widget::common_widget::CommonWidget;
use crate::state::global_state::GlobalState;
use crate::focus::Focusable;

pub trait Event<GS>: CommonWidget<GS> + StateSync<GS> + Focusable<GS> where GS: GlobalState {
    /// A function that will be called when a mouse event occurs.
    /// It will only get called on the events where the cursor is inside.
    /// Return true if the event is consumed, and will thus not be delegated to other
    /// widgets.
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &mut GS);

    /// A function that will get called when a keyboard event occurs.
    /// This event will be given to all widgets, no matter if they are in focus or not.
    /// This is because the focus will be decided by the widgets themselves.
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS);

    /// This will get called if there are event that are not covered by the other functions.
    /// This will get delegated to all widgets.
    /// It will never get called with mouse or keyboard events.
    /// TODO: Separate touch events. And add global state
    fn handle_other_event(&mut self, event: &WidgetEvent);

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &mut GS);

    fn process_mouse_event_default(&mut self, event: &MouseEvent, consumed: &bool, env: &mut Environment<GS>, global_state: &mut GS) {
        self.update_all_widget_state(env, global_state);

        if !*consumed {
            self.handle_mouse_event(event, consumed, env, global_state);
        }

        self.insert_local_state(env);

        for child in self.get_proxied_children() {
            child.process_mouse_event(event, &consumed, env, global_state);
            if *consumed { return () }
        }



        self.update_local_widget_state(env)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS);

    fn process_keyboard_event_default(&mut self, event: &KeyboardEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        self.update_all_widget_state(env, global_state);

        self.handle_keyboard_event(event, env, global_state);

        self.insert_local_state(env);

        for child in self.get_proxied_children() {
            child.process_keyboard_event(event, env, global_state);
        }

        self.update_local_widget_state(env)
    }

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment<GS>, global_state: &mut GS);

    fn process_other_event_default(&mut self, event: &WidgetEvent, env: &mut Environment<GS>, global_state: &mut GS) {
        self.update_all_widget_state(env, global_state);

        self.handle_other_event(event);

        self.insert_local_state(env);

        for child in self.get_proxied_children() {
            child.process_other_event(event, env, global_state);
        }

        self.update_local_widget_state(env)
    }
}