use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::widget::Widget;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct IfElse {
    id: Uuid,
    when_true: Box<dyn Widget>,
    when_false: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state] predicate: BoolState,
}

impl IfElse {
    pub fn new<B: Into<BoolState>>(predicate: B) -> Box<Self> {
        Box::new(IfElse {
            id: Uuid::new_v4(),
            predicate: predicate.into(),
            when_true: Frame::init(0.0, 0.0, Rectangle::initialize(vec![])),
            when_false: Frame::init(0.0, 0.0, Rectangle::initialize(vec![])),
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

impl Layout for IfElse {
    fn flexibility(&self) -> u32 {
        if *self.predicate.value() {
            self.when_true.flexibility()
        } else {
            self.when_false.flexibility()
        }
    }

    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        if *self.predicate.value() {
            self.dimension = self.when_true.calculate_size(requested_size, env);
        } else {
            self.dimension = self.when_false.calculate_size(requested_size, env);
        }
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        if *self.predicate.value() {
            positioning(position, dimension, &mut self.when_true);

            self.when_true.position_children();
        } else {
            positioning(position, dimension, &mut self.when_false);

            self.when_false.position_children();
        }
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
        Flags::EMPTY
    }

    fn children(&self) -> WidgetIter {
        if *self.predicate.value() {
            if self.when_true.flag() == Flags::PROXY {
                self.when_true.children()
            } else {
                WidgetIter::single(self.when_true.deref())
            }
        } else {
            if self.when_false.flag() == Flags::PROXY {
                self.when_false.children()
            } else {
                WidgetIter::single(self.when_false.deref())
            }
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            if self.when_true.flag() == Flags::PROXY {
                self.when_true.children_mut()
            } else {
                WidgetIterMut::single(self.when_true.deref_mut())
            }
        } else {
            if self.when_false.flag() == Flags::PROXY {
                self.when_false.children_mut()
            } else {
                WidgetIterMut::single(self.when_false.deref_mut())
            }
        }
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            WidgetIterMut::single(self.when_true.deref_mut())
        } else {
            WidgetIterMut::single(self.when_false.deref_mut())
        }
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        if *self.predicate.value() {
            WidgetIterMut::single(self.when_true.deref_mut())
        } else {
            WidgetIterMut::single(self.when_false.deref_mut())
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

    fn set_dimension(&mut self, dimensions: Dimension) {
        self.dimension = dimensions
    }
}

impl ChildRender for IfElse {}


impl WidgetExt for IfElse {}