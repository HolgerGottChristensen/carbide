use widget::Id;
use ::{Rect, text};
use graph::Container;
use render::primitive::Primitive;
use render::owned_primitive::OwnedPrimitive;
use position::Dimensions;
use Point;
use widget::common_widget::CommonWidget;

pub trait Render {
    fn layout(&mut self, proposed_size: Dimensions, fonts: &text::font::Map, positioner: &Fn(&mut CommonWidget, Point) -> ());
    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive>;
    fn get_primitives(&self, proposed_size: Dimensions, fonts: &text::font::Map) -> Vec<Primitive>;
}