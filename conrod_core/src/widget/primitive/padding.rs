use uuid::Uuid;
use widget::primitive::Widget;
use ::{Point, Scalar};
use position::Dimensions;
use widget::common_widget::CommonWidget;


use text::font::Map;
use layout::basic_layouter::BasicLayouter;
use widget::render::Render;
use graph::Container;
use Rect;
use render::primitive::Primitive;
use widget::{Id, Rectangle};
use std::ops::Neg;
use widget::primitive::edge_insets::EdgeInsets;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;
use state::state::{StateList};
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use layout::Layout;
use layout::layouter::Layouter;

pub static SCALE: f64 = -1.0;


#[derive(Debug, Clone)]
pub struct Padding<S> {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    position: Point,
    dimension: Dimensions,
    edge_insets: EdgeInsets
}

impl<S> Padding<S> {
    pub fn init(edge_insets: EdgeInsets, child: Box<dyn Widget<S>>) -> Box<Self> {
        Box::new(Padding{
            id: Default::default(),
            child,
            position: [0.0, 0.0],
            dimension: [0.0, 0.0],
            edge_insets
        })
    }
}

impl<S> Event<S> for Padding<S> {
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

impl<S: 'static + Clone> WidgetExt<S> for Padding<S> {}

impl<S> CommonWidget<S> for Padding<S> {
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

impl<S> Layout<S> for Padding<S> {
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

        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S> Render<S> for Padding<S> {

    fn get_primitives(&self, fonts: &Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::rect_outline(Rect::new(self.position, [self.dimension[0], self.dimension[1]]), 1.0));
        let children: Vec<Primitive> = self.child.get_primitives(fonts);
        prims.extend(children);

        return prims;
    }
}