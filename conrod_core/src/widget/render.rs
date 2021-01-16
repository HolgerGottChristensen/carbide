use crate::{Rect, text};
use crate::render::primitive::Primitive;
use crate::widget::Rectangle;
use crate::widget::common_widget::CommonWidget;
use crate::state::global_state::GlobalState;

pub trait Render<U> {
    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive>;
}

pub trait ChildRender {}

impl<T, U: GlobalState> Render<U> for T where T: CommonWidget<U> + ChildRender {
    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = Vec::new();
        prims.extend(Rectangle::<U>::debug_outline(Rect::new(self.get_position(), self.get_dimension()), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}