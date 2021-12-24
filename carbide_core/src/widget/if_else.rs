use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::widget::Widget;
use carbide_derive::Widget;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct IfElse {
    id: Uuid,
    when_true: Box<dyn Widget>,
    when_false: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state]
    predicate: BoolState,
}

impl IfElse {
    pub fn new<B: Into<BoolState>>(predicate: B) -> Box<Self> {
        Box::new(IfElse {
            id: Uuid::new_v4(),
            predicate: predicate.into(),
            when_true: Empty::new(),
            when_false: Empty::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
        })
    }

    pub fn when_true(mut self, when_true: Box<dyn Widget>) -> Box<Self> {
        self.when_true = when_true;
        Box::new(self)
    }

    pub fn when_false(mut self, when_false: Box<dyn Widget>) -> Box<Self> {
        self.when_false = when_false;
        Box::new(self)
    }
}

impl CommonWidget for IfElse {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::PROXY
    }

    fn children(&self) -> WidgetIter {
        if *self.predicate.value() {
            if self.when_true.flag() == Flags::PROXY {
                self.when_true.children()
            } else if self.when_true.flag() == Flags::IGNORE {
                WidgetIter::Empty
            } else {
                WidgetIter::single(&self.when_true)
            }
        } else {
            if self.when_false.flag() == Flags::PROXY {
                self.when_false.children()
            } else if self.when_false.flag() == Flags::IGNORE {
                WidgetIter::Empty
            } else {
                WidgetIter::single(&self.when_false)
            }
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            if self.when_true.flag() == Flags::PROXY {
                self.when_true.children_mut()
            } else if self.when_true.flag() == Flags::IGNORE {
                WidgetIterMut::Empty
            } else {
                WidgetIterMut::single(&mut self.when_true)
            }
        } else {
            if self.when_false.flag() == Flags::PROXY {
                self.when_false.children_mut()
            } else if self.when_false.flag() == Flags::IGNORE {
                WidgetIterMut::Empty
            } else {
                WidgetIterMut::single(&mut self.when_false)
            }
        }
    }

    fn children_direct(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            WidgetIterMut::single(&mut self.when_true)
        } else {
            WidgetIterMut::single(&mut self.when_false)
        }
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            WidgetIterMut::single(&mut self.when_true)
        } else {
            WidgetIterMut::single(&mut self.when_false)
        }
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

impl WidgetExt for IfElse {}
