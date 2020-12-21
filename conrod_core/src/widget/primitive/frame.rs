use uuid::Uuid;
use widget::primitive::Widget;
use ::{Point, Scalar};
use position::Dimensions;
use widget::common_widget::CommonWidget;


use text::font::Map;
use layout::basic_layouter::BasicLayouter;
use widget::render::Render;
use graph::Container;
use ::{Rect, text};
use render::primitive::Primitive;
use widget::{Id, Rectangle};
use std::ops::Neg;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;
use state::state::{StateList};
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use layout::Layout;
use layout::layouter::Layouter;
use state::environment::Environment;

pub static SCALE: f64 = -1.0;


#[derive(Debug, Clone)]
pub struct Frame<S> {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    position: Point,
    dimension: Dimensions
}

impl<S: 'static> Frame<S> {
    pub fn init(width: Scalar, height: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, height]
        })
    }

    pub fn init_width(width: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [width, -1.0]
        })
    }

    pub fn init_height(height: Scalar, child: Box<dyn Widget<S>>) -> Box<Frame<S>> {
        Box::new(Frame{
            id: Default::default(),
            child: Box::new(child),
            position: [0.0,0.0],
            dimension: [-1.0, height]
        })
    }
}

impl<S: 'static + Clone> WidgetExt<S> for Frame<S> {}

impl<S> Event<S> for Frame<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut S) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList, global_state: &mut S) -> StateList {
        let new_state = self.apply_state(state, global_state);

        if self.child.is_inside(event.get_current_mouse_position()) {


            //Then we delegate the event to its children
            let updated_state = self.child.process_mouse_event(event, &consumed, new_state.clone(), global_state);
            return self.apply_state(updated_state, global_state);
        }

        new_state
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        let new_state = self.apply_state(state, global_state);
        let updated_state = self.child.process_keyboard_event(event, new_state, global_state);
        self.apply_state(updated_state, global_state)
    }

    fn get_state(&self, current_state: StateList) -> StateList {
        current_state
    }

    fn apply_state(&mut self, states: StateList, global_state: &S) -> StateList {
        states
    }

    fn sync_state(&mut self, states: StateList, global_state: &S) {
        let applied_state = self.apply_state(states, global_state);
        let new_state = self.get_state(applied_state);

        self.child.sync_state(new_state, global_state);
    }
}

impl<S> CommonWidget<S> for Frame<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Empty
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::Proxy {
            self.child.get_children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::single(&mut self.child)
    }


    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        [self.dimension[0].abs(), self.dimension[1].abs()]
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<S> Layout<S> for Frame<S> {
    fn flexibility(&self) -> u32 {
        9
    }

    fn calculate_size(&mut self, dimension: Dimensions, env: &Environment) -> Dimensions {
        let dimensions = self.dimension;
        let mut abs_dimensions = match (dimensions[0], dimensions[1]) {
            (x, y) if x < 0.0 && y < 0.0 => [dimension[0], dimension[1]],
            (x, y) if x < 0.0 => [dimension[0], self.dimension[1]],
            (x, y) if y < 0.0 => [self.dimension[0], dimension[1]],
            (x, y) => [x, y]
        };

        let child_dimensions = self.child.calculate_size(abs_dimensions, env);

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

impl<S> Render<S> for Frame<S> {

    fn get_primitives(&self, fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::rect_outline(Rect::new(self.position, [self.dimension[0].abs(), self.dimension[1].abs()]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(fonts);
        prims.extend(children);

        return prims;
    }
}