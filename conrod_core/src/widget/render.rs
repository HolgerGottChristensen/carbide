use widget::Id;
use ::{Rect, text};
use graph::Container;
use render::primitive::Primitive;
use render::owned_primitive::OwnedPrimitive;

pub trait Render {
    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive>;
    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive>;
}