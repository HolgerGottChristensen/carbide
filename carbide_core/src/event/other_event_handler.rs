use crate::draw::InnerImageContext;
use crate::environment::{Environment, EnvironmentStack};
use crate::event::Event;
use crate::focus::Focusable;
use crate::state::StateSync;
use crate::text::InnerTextContext;
use crate::widget::{CommonWidget, WidgetSync};

pub trait OtherEventHandler: CommonWidget + WidgetSync + Focusable {
    /// This will get called if there are event that are not covered by the other functions.
    /// This will get delegated to all widgets.
    /// It will never get called with mouse or keyboard events.
    /// TODO: Separate touch events.
    #[allow(unused_variables)]
    fn handle_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {}

    fn process_other_event(&mut self, event: &Event, ctx: &mut OtherEventContext) {
        //if ctx.env.is_event_current() {
            self.sync(ctx.env_stack);
            self.handle_other_event(event, ctx);
        //}

        self.foreach_child_direct(&mut |child| {
            child.process_other_event(event, ctx);
        });
    }
}

pub struct OtherEventContext<'a, 'b: 'a> {
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env: &'a mut Environment,
    pub env_stack: &'a mut EnvironmentStack<'b>,
}