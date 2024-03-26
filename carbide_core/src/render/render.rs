use crate::render::RenderContext;
use crate::state::StateSync;
use crate::widget::{CommonWidget};

/// The render trait is used to get the primitives from a widget. It contains two basic functions.
pub trait Render: CommonWidget + StateSync {
    fn render(&mut self, context: &mut RenderContext) {
        if let Some(cursor) = self.cursor() {
            context.env.set_cursor(cursor);
        }

        self.capture_state(context.env);
        self.foreach_child_mut(&mut |child| {
            child.render(context);
        });
        self.release_state(context.env);
    }
}
