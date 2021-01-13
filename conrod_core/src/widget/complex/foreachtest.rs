use std::fmt::Debug;

use uuid::Uuid;

use crate::Point;
use crate::event::event::NoEvents;
use crate::flags::Flags;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::Layout;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::state::environment::Environment;
use crate::state::state::{GetState, State};
use crate::state::state_sync::StateSync;
use crate::widget::{Rectangle, Text};
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::ChildRender;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};

#[derive(Debug, Clone)]
pub struct ForeachTest<S: Clone + Debug> {
    id: Uuid,
    child: Box<dyn Widget<S>>,
    position: Point,
    dimension: Dimensions,
    index: State<u32, S>,
}

impl<S: 'static + Clone + Debug> ForeachTest<S> {
    pub fn new() -> Box<ForeachTest<S>> {
        Box::new(Self {
            id: Uuid::new_v4(),
            child: Rectangle::initialize(vec![
                Text::initialize(State::new_local("sindex", &"0".to_string()), vec![])
            ]).frame(60.0,30.0),
            position: [100.0,100.0],
            dimension: [100.0,100.0],
            index: State::new_local("index", &(0 as u32))
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

impl<S: Clone + Debug> NoEvents for ForeachTest<S> {}

impl<S: Clone + Debug> StateSync<S> for ForeachTest<S> {
    fn insert_local_state(&self, env: &mut Environment<S>) {
        env.insert_local_state(&State::<String, S>::new_local("sindex", &self.index.get_latest_value().to_string()))
    }

    fn update_all_widget_state(&mut self, env: &Environment<S>, _global_state: &S) {
        self.update_local_widget_state(env)
    }

    fn update_local_widget_state(&mut self, env: &Environment<S>) {
        env.update_local_state(&mut self.index);
    }

    fn sync_state(&mut self, env: &mut Environment<S>, global_state: &S) {
        self.default_sync_state(env, global_state)
    }
}

impl<S: Clone + Debug> ChildRender for ForeachTest<S> {}

impl<S: Clone + Debug> Layout<S> for ForeachTest<S> {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &Environment<S>) -> Dimensions {
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