use event_handler::{MouseEvent, KeyboardEvent, WidgetEvent};
use widget::common_widget::CommonWidget;

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

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool);

    fn process_mouse_event_default(&mut self, event: &MouseEvent, consumed: &bool) {

        // First we handle the event in the widget
        self.handle_mouse_event(event, &consumed);
        if *consumed {return}

        for child in self.get_children_mut().iter_mut() {
            if child.is_inside(event.get_current_mouse_position()) {
                //Then we delegate the event to its children
                child.process_mouse_event(event, &consumed);

                if *consumed {return}
            }
        }
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent);

    fn process_keyboard_event_default(&mut self, event: &KeyboardEvent) {
        // First we handle the event in the widget
        self.handle_keyboard_event(event);

        for child in self.get_children_mut().iter_mut() {

            //Then we delegate the event to its children
            child.process_keyboard_event(event);

        }
    }
}