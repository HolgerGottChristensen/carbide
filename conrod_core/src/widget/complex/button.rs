use uuid::Uuid;
use widget::primitive::CWidget;
use ::{Point, Rect};
use position::Dimensions;
use widget::render::Render;
use render::primitive::Primitive;
use graph::Container;
use widget::{Id, Rectangle};
use text::font::Map;

pub struct NewButton {
    id: Uuid,
    children: Vec<CWidget>,
    position: Point,
    dimension: Dimensions,
}

impl NewButton {
    fn display(&self) -> CWidget {
        Rectangle::new(self.position, self.dimension, vec![])
    }
}

impl Render for NewButton {
    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        unimplemented!()
    }

    fn get_primitives(&self, fonts: &Map) -> Vec<Primitive> {
        let mut prims = Vec::new();
        //prims.extend(self.display().get_primitives());
        //let children: Vec<Primitive> = self.get_children().iter().flat_map(|f| f.get_primitives(fonts)).collect();
        //prims.extend(children);
        prims
    }
}