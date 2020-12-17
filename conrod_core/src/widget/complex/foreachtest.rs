use uuid::Uuid;
use ::{Point, Rect};
use position::Dimensions;
use widget::render::{Render, ChildRender};
use render::primitive::Primitive;
use graph::Container;
use widget::{Id, Rectangle, Oval, HStack, Text};
use text::font::Map;
use widget::common_widget::CommonWidget;
use ::{text, Scalar};
use widget::primitive::Widget;
use widget::primitive::widget::WidgetExt;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use state::state::{StateList, GetState, State};
use daggy::petgraph::graph::node_index;
use render::primitive_kind::PrimitiveKind;

use layout::basic_layouter::BasicLayouter;
use widget::primitive::spacer::{Spacer, SpacerDirection};
use input::Key;
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};
use widget::primitive::foreach::ForEach;
use widget::primitive::v_stack::VStack;
use layout::Layout;
use layout::layouter::Layouter;

#[derive(Debug, Clone)]
pub struct ForeachTest<S> {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    position: Point,
    dimension: Dimensions,
    index: State<u32>
}

impl<S: 'static + Clone> ForeachTest<S> {
    pub fn new() -> Box<ForeachTest<S>> {
        Box::new(Self {
            id: Uuid::new_v4(),
            child: Rectangle::initialize(vec![
                Text::initialize(State::new("sindex", &"0".to_string()), vec![])
            ]).frame(60.0,30.0),
            position: [100.0,100.0],
            dimension: [100.0,100.0],
            index: State::new("index", &(0 as u32))
        })
    }
}

impl<S> CommonWidget<S> for ForeachTest<S> {
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
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl<S> Event<S> for ForeachTest<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        ()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList) -> StateList {
        self.process_mouse_event_default(event, consumed, state)
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList, global_state: &mut S) -> StateList {
        self.process_keyboard_event_default(event, state, global_state)
    }

    fn get_state(&self, mut current_state: StateList) -> StateList {
        current_state.replace_state(State::<String>::new("sindex", &self.index.value.to_string()).into());
        current_state
    }

    fn apply_state(&mut self, states: StateList) -> StateList {
        states.update_local_state(&mut self.index);
        states
    }

    fn sync_state(&mut self, states: StateList) {
        self.sync_state_default(states);
    }
}

impl<S> ChildRender for ForeachTest<S> {}

impl<S> Layout for ForeachTest<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, fonts);
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;
        positioning(position, dimension, &mut self.child);
        self.child.position_children();
    }
}

impl<S: 'static + Clone> WidgetExt<S> for ForeachTest<S> {}