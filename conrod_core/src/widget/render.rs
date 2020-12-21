use widget::{Id, Rectangle};
use ::{Rect, text};
use graph::Container;
use render::primitive::Primitive;
use render::owned_primitive::OwnedPrimitive;
use position::Dimensions;
use Point;
use widget::common_widget::CommonWidget;
use text::font::Map;
use widget::primitive::Widget;
use state::environment::Environment;

pub trait Render<U> {
    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive>;
}

pub trait ChildRender {}

impl<T, U> Render<U> for T where T: CommonWidget<U> + ChildRender {
    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = Vec::new();
        prims.extend(Rectangle::<U>::rect_outline(Rect::new(self.get_position(), self.get_dimension()), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}