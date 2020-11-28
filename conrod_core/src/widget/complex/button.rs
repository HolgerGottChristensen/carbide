use uuid::Uuid;
use ::{Point, Rect};
use position::Dimensions;
use widget::render::Render;
use render::primitive::Primitive;
use graph::Container;
use widget::{Id, Rectangle};
use text::font::Map;
use widget::common_widget::CommonWidget;
use text;
use widget::primitive::Widget;

pub struct NewButton {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    position: Point,
    dimension: Dimensions,
}

impl NewButton {
    fn display(&self) -> Box<Rectangle> {
        Rectangle::new(self.position, self.dimension, vec![])
    }
}

impl Render for NewButton {
    fn layout(&mut self, proposed_size: [f64; 2], fonts:  &text::font::Map, positioner: &dyn Fn(&mut dyn CommonWidget, [f64; 2])) {
        unimplemented!()
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        unimplemented!()
    }

    fn get_primitives(&self, dim: Dimensions, fonts: &Map) -> Vec<Primitive> {
        let mut prims = Vec::new();
        //prims.extend(self.display().get_primitives());
        //let children: Vec<Primitive> = self.get_children().iter().flat_map(|f| f.get_primitives(fonts)).collect();
        //prims.extend(children);
        prims
    }
}