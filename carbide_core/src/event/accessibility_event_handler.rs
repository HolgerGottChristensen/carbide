use accesskit::{Action, ActionData};
use carbide::environment::Environment;
use carbide::focus::Focusable;
use carbide::widget::{CommonWidget, WidgetSync};
use crate::widget::WidgetId;

pub trait AccessibilityEventHandler: CommonWidget + WidgetSync + Focusable {
    #[allow(unused_variables)]
    fn handle_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {}

    fn process_accessibility_event(&mut self, event: &AccessibilityEvent, ctx: &mut AccessibilityEventContext) {
        if *ctx.is_current {
            self.sync(ctx.env);
            self.handle_accessibility_event(event, ctx);
        }

        self.foreach_child_direct(&mut |child| {
            child.process_accessibility_event(event, ctx);
        });
    }
}

pub struct AccessibilityEventContext<'a> {
    pub env: &'a mut Environment,
    pub window_id: &'a u64,
    pub is_current: &'a bool,
}

#[derive(Clone, Debug)]
pub struct AccessibilityEvent {
    pub action: Action,
    pub target: WidgetId,
    pub data: Option<ActionData>
}