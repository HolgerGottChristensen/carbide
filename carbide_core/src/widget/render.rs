use crate::OldRect;
use crate::prelude::{Environment, StateSync};
use crate::render::primitive::Primitive;
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::widget::common_widget::CommonWidget;
use crate::widget::Rectangle;

pub trait Render<GS: GlobalStateContract> {
    fn get_primitives(&mut self, env: &mut Environment<GS>) -> Vec<Primitive>;
}

pub trait RenderProcessor<GS: GlobalStateContract>: CommonWidget<GS> + StateSync<GS> + Render<GS> {
    fn process_get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>);

    fn process_get_primitives_default(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment<GS>, global_state: &GlobalStateContainer<GS>) {
        // Capture the state such that local state will be available to the widget.
        self.capture_state(env, global_state);

        // Get the primitives from the widget. These are added to the resulting primitives.
        primitives.extend(self.get_primitives(env));

        // Release the state such that it is available for the children to capture later.
        self.release_state(env);

        for child in self.get_children_mut() {
            child.process_get_primitives(primitives, env, global_state);
        }
    }
}

pub trait ChildRender {}

impl<T, GS: GlobalStateContract> Render<GS> for T where T: CommonWidget<GS> + ChildRender {
    fn get_primitives(&mut self, _: &mut Environment<GS>) -> Vec<Primitive> {
        /*let mut prims = Vec::new();
        prims.extend(Rectangle::<GS>::debug_outline(OldRect::new(self.get_position(), self.get_dimension()), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(env, global_state)).collect();
        prims.extend(children);*/

        return vec![];
    }
}