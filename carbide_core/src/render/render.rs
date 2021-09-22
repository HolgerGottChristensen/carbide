use crate::prelude::{Environment, StateSync};
use crate::render::primitive::Primitive;
use crate::widget::CommonWidget;

pub trait Render: CommonWidget + StateSync {
    fn get_primitives(&mut self, _primitives: &mut Vec<Primitive>, _env: &mut Environment) {}

    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        // Capture the state such that local state will be available to the widget.
        self.capture_state(env);

        // Get the primitives from the widget. These are added to the resulting primitives.
        self.get_primitives(primitives, env);

        // Release the state such that it is available for the children to capture later.
        self.release_state(env);

        for mut child in self.children_mut() {
            child.process_get_primitives(primitives, env);
        }
    }
}
