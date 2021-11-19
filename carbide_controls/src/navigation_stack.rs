use carbide_core::Widget;
use carbide_core::CommonWidgetImpl;
use carbide_core::draw::{Dimension, Position};
use carbide_core::environment::Environment;
use carbide_core::flags::Flags;
use carbide_core::prelude::TState;
use carbide_core::state::{LocalState, MapState, State, StateExt, StateSync, UsizeState};
use carbide_core::widget::{CommonWidget, CornerRadii, EdgeInsets, HStack, Id, IfElse, Rectangle, RoundedRectangle, Spacer, Text, Widget, WidgetExt, WidgetIter, WidgetIterMut, ZStack};



#[derive(Debug, Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct NavigationStack {
    id: Id,
    position: Position,
    dimension: Dimension,
    stack: TState<Vec<Box<dyn Widget>>>,
    child: TState<Box<dyn Widget>>,
    child_index: UsizeState,
}

impl NavigationStack {
    pub fn empty() -> Box<NavigationStack> {
        todo!()
    }

    pub fn new(stack: TState<Vec<Box<dyn Widget>>>) -> Box<NavigationStack> {

        let child_index = LocalState::new(0);
        let child = stack.index(child_index.clone());
        Box::new(NavigationStack {
            id: Id::new_v4(),
            position: Default::default(),
            dimension: Default::default(),
            stack,
            child,
            child_index,
        })
    }
}

impl CommonWidget for NavigationStack {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::Borrow(self.child.value())
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Borrow(self.child.value_mut())
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Borrow(self.child.value_mut())
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Borrow(self.child.value_mut())
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.dimension = dimension
    }
}

impl StateSync for NavigationStack {
    fn capture_state(&mut self, env: &mut Environment) {
        let i = self.stack.value().len() - 1;
        self.child_index.set_value(i)
    }

    fn release_state(&mut self, env: &mut Environment) {

    }
}

impl WidgetExt for NavigationStack {}