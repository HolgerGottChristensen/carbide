use crate::prelude::{Environment, StateSync};
use crate::render::primitive::Primitive;
use crate::widget::CommonWidget;

/// The render trait is used to get the primitives from a widget. It contains two basic functions.
pub trait Render: CommonWidget + StateSync {

    /// Get the primitives from a widget. You should insert the primitives into the vec.
    /// The default implementation does not add any primitives and is used by most widgets.
    #[allow(unused_variables)]
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {}

    /// The default processing of getting primitives. This is used to add the primitives of self
    /// and all the children of the widget. By default we capture state before getting self's
    /// primitives and release after. We then get recursively call the process of getting
    /// primitives of the children in order.
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
