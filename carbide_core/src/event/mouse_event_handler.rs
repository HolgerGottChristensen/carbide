use crate::draw::InnerImageContext;
use crate::environment::Environment;
use crate::event::MouseEvent;
use crate::focus::Focusable;
use crate::state::StateSync;
use crate::text::InnerTextContext;
use crate::widget::CommonWidget;

pub trait MouseEventHandler: CommonWidget + StateSync + Focusable {
    /// A function that will be called when a mouse event occurs.
    /// It will only get called on the events where the cursor is inside.
    /// Return true if the event is consumed, and will thus not be delegated to other
    /// widgets.
    #[allow(unused_variables)]
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, ctx: &mut MouseEventContext) {}

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, ctx: &mut MouseEventContext) {
        if ctx.env.is_event_current() {
            if !*consumed {
                self.capture_state(ctx.env);
                self.handle_mouse_event(event, consumed, ctx);
                self.release_state(ctx.env);
            }
        }

        self.foreach_child_direct(&mut |child| {
            child.process_mouse_event(event, &consumed, ctx);
            if *consumed {
                return;
            }
        });
    }
}

// TODO: Consider changing to Event Context
pub struct MouseEventContext<'a> {
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env: &'a mut Environment,
}
