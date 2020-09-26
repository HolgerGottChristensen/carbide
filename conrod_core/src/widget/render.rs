use widget::Id;
use Rect;
use graph::Container;
use render::primitive::Primitive;

pub trait Render {
    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive>;
}