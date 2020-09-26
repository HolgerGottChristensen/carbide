use widget::Id;
use Rect;
use graph::Container;
use render::primitive::Primitive;
use render::owned_primitive::OwnedPrimitive;

pub trait Render {
    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive>;
    fn get_primitives(&self) -> Vec<Primitive>;
}