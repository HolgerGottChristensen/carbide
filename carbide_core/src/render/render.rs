use crate::environment::Environment;
use crate::render::primitive::Primitive;
use crate::render::RenderContext;
use crate::state::StateSync;
use crate::widget::{CommonWidget};

/// The render trait is used to get the primitives from a widget. It contains two basic functions.
pub trait Render: CommonWidget + StateSync {
    fn render(&mut self, context: &mut RenderContext, env: &mut Environment) {
        self.capture_state(env);
        self.foreach_child_mut(&mut |child| {
            child.render(context, env);
        });
        self.release_state(env);
    }
}
