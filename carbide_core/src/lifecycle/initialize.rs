use carbide::environment::Environment;
use carbide::widget::{CommonWidget, WidgetSync};
use std::any::Any;

pub trait Initialize: CommonWidget + WidgetSync {
    #[allow(unused_variables)]
    fn initialize(&mut self, ctx: &mut InitializationContext) {}

    fn process_initialization(&mut self, ctx: &mut InitializationContext) {
        self.sync(ctx.env);
        self.initialize(ctx);

        self.foreach_child_direct(&mut |child| {
            child.process_initialization(ctx);
        });
    }
}


pub struct InitializationContext<'a> {
    pub env: &'a mut Environment,
    pub lifecycle_manager: &'a dyn Any,
}