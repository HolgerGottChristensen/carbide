use crate::render::RenderContext;
use crate::state::StateSync;
use crate::widget::{CommonWidget, WidgetSync};

/// The render trait is used to get the primitives from a widget. It contains two basic functions.
pub trait Render: CommonWidget + WidgetSync {
    fn render(&mut self, context: &mut RenderContext) {
        if let Some(cursor) = self.cursor() {
            context.env.set_cursor(cursor);
        }

        self.sync(context.env);
        self.foreach_child_mut(&mut |child| {
            child.render(context);
        });
    }
}
