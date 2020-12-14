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
use state::state::{StateList};
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};

pub static SCALE: f64 = -1.0;


#[derive(Debug, Clone)]
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

impl<S> Event<S> for Frame {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList) -> StateList {
        let new_state = self.apply_state(state);

        if self.child.is_inside(event.get_current_mouse_position()) {


            //Then we delegate the event to its children
            let updated_state = self.child.process_mouse_event(event, &consumed, new_state.clone());
            return self.apply_state(updated_state);
        }

        new_state
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        let new_state = self.apply_state(state);
        let updated_state = self.child.process_keyboard_event(event, new_state);
        self.apply_state(updated_state)
    }

    fn get_state(&self, current_state: StateList) -> StateList {
        current_state
    }

    fn apply_state(&mut self, states: StateList) -> StateList {
        states
    }

    fn sync_state(&mut self, states: StateList) {
        let applied_state = self.apply_state(states);
        let new_state = self.get_state(applied_state);

        self.child.sync_state(new_state);
    }
}

impl CommonWidget for Frame {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
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

    fn get_primitives(&self, fonts: &Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, [self.dimension[0].abs(), self.dimension[1].abs()]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(fonts);
        prims.extend(children);

        return prims;
    }
}