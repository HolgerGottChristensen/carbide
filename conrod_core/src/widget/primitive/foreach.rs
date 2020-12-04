use uuid::Uuid;
use ::{Point, Rect};
use position::Dimensions;
use widget::render::{Render, ChildRender};
use render::primitive::Primitive;
use graph::Container;
use widget::{Id, Rectangle, HStack, Text};
use text::font::Map;
use widget::common_widget::CommonWidget;
use ::{text, Scalar};
use widget::primitive::Widget;
use widget::primitive::widget::{WidgetExt, CloneableWidget};
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use state::state::{StateList, DefaultState, GetState, State};
use daggy::petgraph::graph::node_index;
use render::primitive_kind::PrimitiveKind;
use widget::layout::Layout;
use layout::basic_layouter::BasicLayouter;
use widget::primitive::spacer::{Spacer, SpacerDirection};
use input::Key;
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};

#[derive(Debug)]
pub struct ForEach {
    id: Uuid,
    children: Vec<Box<dyn Widget>>,
    delegate: Box<dyn CloneableWidget>,
    number: State<u32>,
    position: Point,
    dimension: Dimensions
}

impl ForEach {
    pub fn new(number: State<u32>, delegate: Box<dyn CloneableWidget>) -> Box<ForEach> {
        Box::new(Self {
            id: Uuid::new_v4(),
            children: vec![],
            delegate,
            number,
            position: [100.0,100.0],
            dimension: [100.0,100.0]
        })
    }
}

impl CommonWidget for ForEach {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::Proxy
    }

    fn get_children(&self) -> WidgetIter {
        self.children
            .iter()
            .rfold(WidgetIter::Empty, |acc, x| {
                if x.get_flag() == Flags::Proxy {
                    WidgetIter::Multi(Box::new(x.get_children()), Box::new(acc))
                } else {
                    WidgetIter::Single(x, Box::new(acc))
                }
            })
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        self.children
            .iter_mut()
            .rfold(WidgetIterMut::Empty, |acc, x| {
                if x.get_flag() == Flags::Proxy {
                    WidgetIterMut::Multi(Box::new(x.get_children_mut()), Box::new(acc))
                } else {
                    WidgetIterMut::Single(x, Box::new(acc))
                }
            })
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        self.children.iter_mut()
            .filter(|s| s.get_flag() == Flags::Proxy)
            .rfold(WidgetIterMut::Empty, |acc, x| {
                WidgetIterMut::Single(x, Box::new(acc))
            })
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

impl Event for ForEach {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        unimplemented!()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        unimplemented!()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, state: StateList<DefaultState>) -> StateList<DefaultState> {
        println!("Foreach mouseevent");
        state
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent, state: StateList<DefaultState>) -> StateList<DefaultState> {
        self.apply_state(state)
    }

    fn get_state(&self, mut current_state: StateList<DefaultState>) -> StateList<DefaultState> {
        unimplemented!()
    }

    fn apply_state(&mut self, states: StateList<DefaultState>) -> StateList<DefaultState> {
        match states.get_state(&self.number.id) {
            None => (),
            Some(v) => {
                self.text = v.clone().into()
            }
        }
        states
    }

    fn sync_state(&mut self, states: StateList<DefaultState>) {
        unimplemented!()
    }
}

impl ChildRender for ForEach {}

impl Layout for ForEach {
    fn flexibility(&self) -> u32 {
        unimplemented!()
    }

    fn calculate_size(&mut self, requested_size: Dimensions, fonts: &Map) -> Dimensions {
        unimplemented!()
    }

    fn position_children(&mut self) {
        unimplemented!()
    }
}

impl WidgetExt for ForEach {}