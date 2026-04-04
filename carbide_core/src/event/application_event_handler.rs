use crate::draw::{Dimension, ImageContext, Position, Scalar};
use crate::environment::{Environment};
use crate::text::TextContext;
use crate::widget::{CommonWidget, WidgetSync};

pub trait ApplicationEventHandler: CommonWidget + WidgetSync {
    #[allow(unused_variables)]
    fn handle_application_event(&mut self, event: &ApplicationEvent, ctx: &mut ApplicationEventContext) {}

    fn process_application_event(&mut self, event: &ApplicationEvent, ctx: &mut ApplicationEventContext) {
        self.sync(ctx.env);
        self.handle_application_event(event, ctx);

        self.foreach_child_mut(&mut |child| {
            child.process_application_event(event, ctx);
        });
    }
}


pub struct ApplicationEventContext<'a, 'b: 'a> {
    pub text: &'a mut dyn TextContext,
    pub image: &'a mut dyn ImageContext,
    pub env: &'a mut Environment<'b>
}


#[derive(Clone, Debug, PartialEq)]
pub enum ApplicationEvent {
    Resumed,
    Suspended,
    Exited
}