use uuid::Uuid;
use widget::primitive::Widget;
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
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;

pub static SCALE: f64 = -1.0;


#[derive(Debug)]
pub struct Frame {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Point,
    dimension: Dimensions
}

impl Frame {
    pub fn init(width: Scalar, height: Scalar, child: Box<dyn Widget>) -> Box<Frame> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, height]
        })
    }

    pub fn init_width(width: Scalar, child: Box<dyn Widget>) -> Box<Frame> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, -1.0]
        })
    }

    pub fn init_height(height: Scalar, child: Box<dyn Widget>) -> Box<Frame> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [-1.0, height]
        })
    }
}

impl WidgetExt for Frame {}

impl Event for Frame {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        unimplemented!()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        if self.child.is_inside(event.get_current_mouse_position()) {
            //Then we delegate the event to its children
            self.child.process_mouse_event(event, &consumed);
        }
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent) {
        self.child.process_keyboard_event(event);
    }
}

impl CommonWidget for Frame {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_children(&self) -> &Vec<Box<dyn Widget>> {
        unimplemented!()
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
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

impl Layout for Frame {
    fn flexibility(&self) -> u32 {
        9
    }

    fn calculate_size(&mut self, dimension: Dimensions, fonts: &Map) -> Dimensions {
        let dimensions = self.dimension;
        let mut abs_dimensions = match (dimensions[0], dimensions[1]) {
            (x, y) if x < 0.0 && y < 0.0 => [dimension[0], dimension[1]],
            (x, y) if x < 0.0 => [dimension[0], self.dimension[1]],
            (x, y) if y < 0.0 => [self.dimension[0], dimension[1]],
            (x, y) => [x, y]
        };
        let child_dimensions = self.child.calculate_size(abs_dimensions, fonts);

        if dimensions[0] < 0.0 {
            self.dimension = [child_dimensions[0].abs().neg(), dimensions[1]]
        }

        if dimensions[1] < 0.0 {
            self.dimension = [self.dimension[0], child_dimensions[1].abs().neg()]
        }

        [self.dimension[0].abs(), self.dimension[1].abs()]
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = [self.dimension[0].abs(), self.dimension[1].abs()];

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl Render for Frame {
    fn layout(&mut self, proposed_size: [f64; 2], fonts: &Map, positioner: &dyn Fn(&mut dyn CommonWidget, [f64; 2])) {
        unimplemented!()
    }

    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        unimplemented!()
    }

    fn get_primitives(&self, proposed_size: Dimensions, fonts: &Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, [self.dimension[0].abs(), self.dimension[1].abs()]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(proposed_size, fonts);
        prims.extend(children);

        return prims;
    }
}