use crate::Rect;
use crate::render::primitive::Primitive;
use crate::widget::Rectangle;
use crate::widget::common_widget::CommonWidget;
use crate::state::global_state::GlobalState;
use crate::prelude::Environment;

pub trait Render<GS: GlobalState> {
    fn get_primitives(&mut self, env: &Environment<GS>, global_state: &GS) -> Vec<Primitive>;
}

pub trait ChildRender {}

impl<T, GS: GlobalState> Render<GS> for T where T: CommonWidget<GS> + ChildRender {
    fn get_primitives(&mut self, env: &Environment<GS>, global_state: &GS) -> Vec<Primitive> {
        let mut prims = Vec::new();
        prims.extend(Rectangle::<GS>::debug_outline(Rect::new(self.get_position(), self.get_dimension()), 1.0));
        let children: Vec<Primitive> = self.get_children_mut().flat_map(|f| f.get_primitives(env, global_state)).collect();
        prims.extend(children);

        return prims;
    }
}