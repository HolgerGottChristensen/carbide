use std::rc::Rc;

use crate::draw::{Dimension, Position};
use crate::state::{LocalState, ReadState, State, TState, ValueCell};
use crate::widget::{CommonWidget, Widget, WidgetExt, WidgetId, WidgetIter, WidgetIterMut};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Overlay {
    id: WidgetId,
    child: Rc<ValueCell<Box<dyn Widget>>>,
    #[state]
    showing: TState<bool>,
    position: TState<Position>,
    dimension: TState<Dimension>,
}

impl Overlay {
    // We do not need to return this in a box, because the overlay widgets should only
    pub fn new(child: Box<dyn Widget>) -> Self {
        Overlay {
            id: WidgetId::new(),
            child: Rc::new(ValueCell::new(child)),
            showing: LocalState::new(false),
            position: LocalState::new(Position::new(0.0, 0.0)),
            dimension: LocalState::new(Dimension::new(100.0, 100.0)),
        }
    }

    pub fn showing(mut self, showing: impl Into<TState<bool>>) -> Self {
        self.showing = showing.into();
        self
    }

    pub fn is_showing(&self) -> bool {
        *self.showing.value()
    }

    pub fn set_showing(&mut self, val: bool) {
        self.showing.set_value(val);
    }
}

impl CommonWidget for Overlay {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::borrow(self.child.borrow())
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::borrow(self.child.borrow_mut())
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::borrow(self.child.borrow_mut())
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::borrow(self.child.borrow_mut())
    }

    fn position(&self) -> Position {
        *self.position.value()
    }

    fn set_position(&mut self, position: Position) {
        *self.position.value_mut() = position;
    }

    fn dimension(&self) -> Dimension {
        *self.dimension.value()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        *self.dimension.value_mut() = dimension;
    }
}

impl WidgetExt for Overlay {}
