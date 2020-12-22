use std::fmt::Debug;
use std::ops::Deref;

use daggy::petgraph::graph::node_index;
use uuid::Uuid;

use ::{Point, Rect};
use ::{Scalar, text};
use event::event::Event;
use event_handler::{KeyboardEvent, MouseEvent, WidgetEvent};
use flags::Flags;
use graph::Container;
use input::Key;
use layout::basic_layouter::BasicLayouter;
use layout::Layout;
use layout::layouter::Layouter;
use position::Dimensions;
use render::primitive::Primitive;
use render::primitive_kind::PrimitiveKind;
use state::environment::Environment;
use state::state::{GetState, LocalStateList, State};
use state::state_sync::StateSync;
use text::font::Map;
use widget::{HStack, Id, Oval, Rectangle, Text};
use widget::common_widget::CommonWidget;
use widget::primitive::foreach::ForEach;
use widget::primitive::spacer::{Spacer, SpacerDirection};
use widget::primitive::v_stack::VStack;
use widget::primitive::Widget;
use widget::primitive::widget::WidgetExt;
use widget::render::{ChildRender, Render};
use widget::widget_iterator::{WidgetIter, WidgetIterMut};

#[derive(Debug, Clone)]
pub struct ForeachTest<S: Clone + Debug> {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    position: Point,
    dimension: Dimensions,
    index: State<u32, S>
}

impl<S: 'static + Clone + Debug> ForeachTest<S> {
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

impl<S: Clone + Debug> CommonWidget<S> for ForeachTest<S> {
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

impl<S: Clone + Debug> Event<S> for ForeachTest<S> {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool, global_state: &mut S) {
        ()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent, global_state: &mut S) {
        ()
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        ()
    }
}

impl<S: Clone + Debug> StateSync<S> for ForeachTest<S> {
    fn insert_local_state(&self, env: &mut Environment) {
        env.insert_local_state(&State::<String, S>::new("sindex", &self.index.get_latest_value().to_string()))
    }

    fn update_all_widget_state(&mut self, env: &Environment, global_state: &S) {
        self.update_local_widget_state(env)
    }

    fn update_local_widget_state(&mut self, env: &Environment) {
        env.update_local_state(&mut self.index);
    }
}

impl<S: Clone + Debug> ChildRender for ForeachTest<S> {}

impl<S: Clone + Debug> Layout<S> for ForeachTest<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment) -> Dimensions {
        self.dimension = self.child.calculate_size(requested_size, env);
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

impl<S: 'static + Clone + Debug> WidgetExt<S> for ForeachTest<S> {}