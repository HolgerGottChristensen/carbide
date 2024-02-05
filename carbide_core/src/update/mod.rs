use crate::draw::InnerImageContext;
use crate::environment::Environment;
use crate::state::StateSync;
use crate::text::InnerTextContext;
use crate::widget::CommonWidget;

pub trait Update: CommonWidget + StateSync {
    #[allow(unused_variables)]
    fn update(&mut self, ctx: &mut UpdateContext) {}

    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.capture_state(ctx.env);
        self.update(ctx);
        self.release_state(ctx.env);

        self.foreach_child_direct(&mut |child| {
            child.process_update(ctx);
        });
    }
}


pub struct UpdateContext<'a> {
    pub text: &'a mut dyn InnerTextContext,
    pub image: &'a mut dyn InnerImageContext,
    pub env: &'a mut Environment,
}