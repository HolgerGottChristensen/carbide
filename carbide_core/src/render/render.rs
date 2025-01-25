use crate::misc::cursor::MouseCursor;
use crate::render::RenderContext;
use crate::widget::{CommonWidget, WidgetSync};

/// The render trait is used to get the primitives from a widget. It contains two basic functions.
pub trait Render: CommonWidget + WidgetSync {
    fn render(&mut self, ctx: &mut RenderContext) {
        if let Some(cursor) = self.cursor() {
            if let Some(env_cursor) = ctx.env_stack.get_mut::<MouseCursor>() {
                *env_cursor = cursor;
            }
        }

        self.sync(ctx.env_stack);
        self.foreach_child_mut(&mut |child| {
            child.render(ctx);
        });
    }
}
