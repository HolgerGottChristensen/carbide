use widget::{Id, Rectangle};
use ::{Rect, text};
use graph::Container;
use render::primitive::Primitive;
use render::owned_primitive::OwnedPrimitive;
use position::Dimensions;
use Point;
use widget::common_widget::CommonWidget;
use text::font::Map;

pub trait Render {
    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive>;
}

pub trait ChildRender {}

impl<T> Render for T where T: ChildRender + CommonWidget {
    fn get_primitives(&self, fonts: &Map) -> Vec<Primitive> {
        let mut prims = Vec::new();
        prims.extend(Rectangle::rect_outline(Rect::new(self.get_position(), self.get_dimension()), 1.0));
        let children: Vec<Primitive> = self.get_children().flat_map(|f| f.get_primitives(fonts)).collect();
        prims.extend(children);

        return prims;
    }
}