use event_handler::{MouseEvent, KeyboardEvent, WidgetEvent};
use widget::common_widget::CommonWidget;
use state::state::{StateList, DefaultState};

pub trait Event: CommonWidget {
    /// A function that will be called when a mouse event occurs.
    /// It will only get called on the events where the cursor is inside.
    /// Return true if the event is consumed, and will thus not be delegated to other
    /// widgets.
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool);

    /// A function that will get called when a keyboard event occurs.
    /// This event will be given to all widgets, no matter if they are in focus or not.
    /// This is because the focus will be decided by the widgets themselves.
    fn handle_keyboard_event(&mut self, event: &KeyboardEvent);

    /// This will get called if there are event that are not covered by the other functions.
    /// This will get delegated to all widgets.
    /// It will never get called with mouse or keyboard events.
    /// TODO: Separate touch events.
    fn handle_other_event(&mut self, event: &WidgetEvent);

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList<DefaultState>) -> StateList<DefaultState>;

    fn process_mouse_event_default(&mut self, event: &MouseEvent, consumed: &bool, state: StateList<DefaultState>) -> StateList<DefaultState> {

        // Apply state from its parent
        let new_state = self.apply_state(state);

        // First we handle the event in the widget
        self.handle_mouse_event(event, &consumed);
        if *consumed {return new_state}

        // Add the state from itself, to the state list
        let mut state_for_children = self.get_state(new_state);

        for child in self.get_children_mut(){
            if child.is_inside(event.get_current_mouse_position()) {
                //Then we delegate the event to its children
                state_for_children = child.process_mouse_event(event, &consumed, state_for_children);

                if *consumed {return state_for_children}
            }
        }

        // We then apply the changed state from its children, to save it for itself.
        self.apply_state(state_for_children)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList<DefaultState>) -> StateList<DefaultState>;

    fn process_keyboard_event_default(&mut self, event: &KeyboardEvent, state: StateList<DefaultState>) -> StateList<DefaultState>{

        // Apply state from its parent
        let new_state = self.apply_state(state);

        // First we handle the event in the widget
        self.handle_keyboard_event(event);

        // Add the state from itself, to the state list
        let mut state_for_children = self.get_state(new_state);

        for child in self.get_children_mut() {

            // Then we delegate the event to its children, we also makes sure to update
            // current state for the next child
            state_for_children = child.process_keyboard_event(event, state_for_children);

        }
        // We then apply the changed state from its children, to save it for itself.
        self.apply_state(state_for_children)
    }

    fn get_state(&self, current_state: StateList<DefaultState>) -> StateList<DefaultState>;

    fn apply_state(&mut self, states: StateList<DefaultState>) -> StateList<DefaultState>;

    fn sync_state(&mut self, states: StateList<DefaultState>);

    fn sync_state_default(&mut self, states: StateList<DefaultState>) {
        let applied_state = self.apply_state(states);
        let new_state = self.get_state(applied_state);

        for child in self.get_children_mut() {
            child.sync_state(new_state.clone())
        }
    }
}