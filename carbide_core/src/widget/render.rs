use crate::OldRect;
use crate::prelude::{Environment, StateSync};
use crate::render::primitive::Primitive;
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::widget::common_widget::CommonWidget;
use crate::widget::Rectangle;

pub trait Render {
    fn get_primitives(&mut self, env: &mut Environment) -> Vec<Primitive>;
}

pub trait RenderProcessor: CommonWidget + StateSync + Render {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment);

    fn process_get_primitives_default(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        // Capture the state such that local state will be available to the widget.
        self.capture_state(env);

        // Get the primitives from the widget. These are added to the resulting primitives.
        primitives.extend(self.get_primitives(env));

        // Release the state such that it is available for the children to capture later.
        self.release_state(env);

        for child in self.get_children_mut() {
            child.process_get_primitives(primitives, env);
        }
    }
}

pub trait ChildRender {}

impl<T> Render for T where T: CommonWidget + ChildRender {
    fn get_primitives(&mut self, _: &mut Environment) -> Vec<Primitive> {
        /*let mut prims = Vec::new();
        prims.extend(Rectangle::<GS>::debug_outline(OldRect::new(self.get_position(), self.get_dimension()), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(env, global_state)).collect();
        prims.extend(children);*/

        return vec![];
    }
}