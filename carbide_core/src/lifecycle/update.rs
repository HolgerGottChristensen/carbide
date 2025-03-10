use crate::draw::ImageContext;
use crate::environment::{Environment};
use crate::text::TextContext;
use crate::widget::{CommonWidget, WidgetSync};

pub trait Update: CommonWidget + WidgetSync {
    #[allow(unused_variables)]
    fn update(&mut self, ctx: &mut UpdateContext) {}

    fn process_update(&mut self, ctx: &mut UpdateContext) {
        self.sync(ctx.env);
        self.update(ctx);

        self.foreach_child_direct(&mut |child| {
            child.process_update(ctx);
        });
    }
}


pub struct UpdateContext<'a, 'b: 'a> {
    pub text: &'a mut dyn TextContext,
    pub image: &'a mut dyn ImageContext,
    pub env: &'a mut Environment<'b>,
}