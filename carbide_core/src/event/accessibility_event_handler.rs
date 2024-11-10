use accesskit::{Action, ActionData};
use carbide::environment::Environment;
use carbide::focus::Focusable;
use carbide::widget::{CommonWidget, WidgetSync};
use crate::accessibility::AccessibilityAction;
use crate::environment::EnvironmentStack;
use crate::widget::WidgetId;

pub trait AccessibilityEventHandler: CommonWidget + WidgetSync + Focusable {
    #[allow(unused_variables)]
    fn handle_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {}

    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        if event.target == self.id() {
            self.sync(ctx.env_stack);
            self.handle_accessibility_event(event, ctx);
        } else {
            self.foreach_child_direct(&mut |child| {
                child.process_accessibility_event(event, ctx);
            });
        }
    }
}

pub struct AccessibilityEventContext<'a, 'b: 'a> {
    pub env: &'a mut Environment,
    pub env_stack: &'a mut EnvironmentStack<'b>,
}

#[derive(Clone, Debug)]
pub struct AccessibilityEvent<'a> {
    pub action: AccessibilityAction,
    pub target: WidgetId,
    pub data: &'a Option<ActionData>
}