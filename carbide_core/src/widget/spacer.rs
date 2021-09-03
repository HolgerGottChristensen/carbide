use crate::draw::{Dimension, Position};
use crate::prelude::*;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout)]
pub struct Spacer {
    id: Uuid,
    position: Position,
    dimension: Dimension,
}

impl Spacer {
    pub fn new() -> Box<Self> {
        Box::new(Spacer {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

impl Layout for Spacer {
    fn calculate_size(&mut self, requested_size: Dimension, _: &mut Environment) -> Dimension {
        self.dimension = requested_size;
        requested_size
    }
}

impl CommonWidget for Spacer {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::SPACER
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
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

impl WidgetExt for Spacer {}
