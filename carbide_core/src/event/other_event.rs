use crate::environment::Environment;
use crate::event::WidgetEvent;
use crate::focus::Focusable;
use crate::state::StateSync;
use crate::widget::CommonWidget;

pub trait OtherEventHandler: CommonWidget + StateSync + Focusable {
    /// This will get called if there are event that are not covered by the other functions.
    /// This will get delegated to all widgets.
    /// It will never get called with mouse or keyboard events.
    /// TODO: Separate touch events.
    fn handle_other_event(&mut self, _event: &WidgetEvent, _env: &mut Environment) {}

    fn process_other_event(&mut self, event: &WidgetEvent, env: &mut Environment) {
        self.capture_state(env);
        self.handle_other_event(event, env);
        self.release_state(env);

        for child in self.children_direct() {
            child.process_other_event(event, env);
        }
    }
}
