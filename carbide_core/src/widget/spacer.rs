use crate::draw::{Dimension, Position};
use crate::prelude::*;

#[derive(Clone, Debug, Widget)]
#[carbide_exclude(Layout)]
pub struct Spacer {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
}

impl Spacer {
    pub fn new() -> Box<Self> {
        Box::new(Spacer {
            id: WidgetId::new(),
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
    fn id(&self) -> WidgetId {
        self.id
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

    fn children_direct(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn children_direct_rev(&mut self) -> WidgetIterMut {
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
