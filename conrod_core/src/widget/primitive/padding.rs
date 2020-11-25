use uuid::Uuid;
use widget::primitive::CWidget;
use ::{Point, Scalar};
use position::Dimensions;
use widget::common_widget::CommonWidget;
use widget::envelope_editor::EnvelopePoint;
use widget::layout::Layout;
use text::font::Map;
use layout::basic_layouter::BasicLayouter;
use widget::render::Render;
use graph::Container;
use Rect;
use render::primitive::Primitive;
use widget::{Id, Rectangle};
use std::ops::Neg;
use widget::primitive::edge_insets::EdgeInsets;

pub static SCALE: f64 = -1.0;


#[derive(Clone, Debug)]
pub struct Padding {
    id: Uuid,
    child: Box<CWidget>,
    position: Point,
    dimension: Dimensions,
    edge_insets: EdgeInsets
}

impl Padding {
    pub fn init(edge_insets: EdgeInsets, child: CWidget) -> CWidget {
        CWidget::Padding(Padding{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            edge_insets
        })
    }
}

impl CommonWidget for Padding {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_children(&self) -> &Vec<CWidget> {
        unimplemented!()
    }

    fn get_position(&self) -> [f64; 2] {
        unimplemented!()
    }

    fn get_x(&self) -> Scalar {
        self.position[0]
    }

    fn set_x(&mut self, x: f64) {
        self.position = Point::new(x, self.position.get_y());
    }

    fn get_y(&self) -> Scalar {
        self.position[1]
    }

    fn set_y(&mut self, y: f64) {
        self.position = Point::new(self.position.get_x(), y);
    }

    fn get_size(&self) -> [f64; 2] {
        unimplemented!()
    }

    fn get_width(&self) -> Scalar {
        self.dimension[0].abs()
    }

    fn get_height(&self) -> Scalar {
        self.dimension[1].abs()
    }
}

impl Layout for Padding {
    fn flexibility(&self) -> u32 {
        9
    }

    fn calculate_size(&mut self, dimension: Dimensions, fonts: &Map) -> Dimensions {
        let dimensions = [dimension[0] - self.edge_insets.left - self.edge_insets.right, dimension[1] - self.edge_insets.top - self.edge_insets.bottom];

        let child_dimensions = self.child.calculate_size(dimensions, fonts);

        self.dimension = [child_dimensions[0] + self.edge_insets.left + self.edge_insets.right, child_dimensions[1] + self.edge_insets.top + self.edge_insets.bottom];

        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = [self.position[0] + self.edge_insets.left, self.position[1] + self.edge_insets.top];
        let dimension = [self.dimension[0] - self.edge_insets.left - self.edge_insets.right, self.dimension[1] - self.edge_insets.top - self.edge_insets.bottom];

        positioning(position, dimension, &mut *self.child);
        self.child.position_children();
    }
}

impl Render for Padding {
    fn layout(&mut self, proposed_size: [f64; 2], fonts: &Map, positioner: &dyn Fn(&mut dyn CommonWidget, [f64; 2])) {
        unimplemented!()
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        unimplemented!()
    }

    fn get_primitives(&self, proposed_size: Dimensions, fonts: &Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, [self.dimension[0], self.dimension[1]]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(proposed_size, fonts);
        prims.extend(children);

        return prims;
    }
}