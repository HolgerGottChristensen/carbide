use std::rc::Rc;

use crate::draw::{Dimension, Position};
use crate::prelude::*;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Overlay {
    id: Uuid,
    child: Rc<ValueCell<Box<dyn Widget>>>,
    #[state] showing: BoolState,
    position: PositionState,
    dimension: DimensionState,
}

impl Overlay {
    // We do not need to return this in a box, because the overlay widgets should only
    pub fn new(child: Box<dyn Widget>) -> Self {
        Overlay {
            id: Uuid::new_v4(),
            child: Rc::new(ValueCell::new(child)),
            showing: LocalState::new(false).into(),
            position: LocalState::new(Position::new(0.0, 0.0)).into(),
            dimension: LocalState::new(Dimension::new(100.0, 100.0)).into(),
        }
    }

    pub fn showing<S: Into<BoolState>>(mut self, showing: S) -> Self {
        self.showing = showing.into();
        self
    }

    pub fn is_showing(&self) -> bool {
        *self.showing.value()
    }

    pub fn set_showing(&mut self, val: bool) {
        *self.showing.value_mut() = val;
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
