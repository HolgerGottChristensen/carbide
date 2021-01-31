use std::fmt::Debug;

use uuid::Uuid;

use crate::Point;
use crate::flags::Flags;
use crate::layout::basic_layouter::BasicLayouter;
use crate::layout::Layout;
use crate::layout::layouter::Layouter;
use crate::position::Dimensions;
use crate::state::environment::Environment;
use crate::state::state::CommonState;
use crate::widget::{Rectangle, Text};
use crate::widget::common_widget::CommonWidget;
use crate::widget::primitive::Widget;
use crate::widget::primitive::widget::WidgetExt;
use crate::widget::render::ChildRender;
use crate::widget::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::color::RED;
use crate::state::global_state::GlobalState;

#[derive(Debug, Clone, Widget)]
#[state_sync(insert_local_state)]
pub struct ForeachTest<GS> where GS: GlobalState {
    id: Uuid,
    child: Box<dyn Widget<GS>>,
    position: Point,
    dimension: Dimensions,
    #[state] index: CommonState<u32, GS>,
}

impl<S: GlobalState> ForeachTest<S> {
    pub fn new() -> Box<ForeachTest<S>> {
        Box::new(Self {
            id: Uuid::new_v4(),
            child: Rectangle::initialize(vec![
                Text::initialize(CommonState::new_local("sindex", &"0".to_string()))
            ]).fill(RED).frame(60.0.into(),30.0.into()),
            position: [100.0,100.0],
            dimension: [100.0,100.0],
            index: CommonState::new_local("index", &(0 as u32))
        })
    }

    fn insert_local_state(&self, env: &mut Environment<S>) {
        env.insert_local_state(&CommonState::<String, S>::new_local("sindex", &self.index.get_latest_value().to_string()));
    }
}

impl<S: GlobalState> CommonWidget<S> for ForeachTest<S> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter<S> {
        if self.child.get_flag() == Flags::PROXY {
            self.child.get_children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        if self.child.get_flag() == Flags::PROXY {
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

impl<S: GlobalState> ChildRender for ForeachTest<S> {}

impl<S: GlobalState> Layout<S> for ForeachTest<S> {
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