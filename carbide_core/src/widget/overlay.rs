use crate::draw::{Dimension, Position};
use crate::prelude::*;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Overlay {
    id: Uuid,
    child: Box<dyn Widget>,
    showing: BoolState,
    position: Position,
    dimension: Dimension,
}

impl Overlay {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Overlay {
            id: Uuid::new_v4(),
            child,
            showing: LocalState::new(false).into(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }

    pub fn showing<S: Into<BoolState>>(mut self, showing: S) -> Box<Self> {
        self.showing = showing.into();
        Box::new(self)
    }

    pub fn is_showing(&self) -> bool {
        *self.showing.value()
    }

    pub fn set_showing(&mut self, val: bool) {
        *self.showing.value_mut() = val;
    }
}

impl CommonWidget for Overlay {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn children(&self) -> WidgetIter {
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(&self.child)
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(&mut self.child)
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(&mut self.child)
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

impl WidgetExt for Overlay {}
