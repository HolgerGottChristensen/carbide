use crate::draw::{Dimension, Position};
use crate::prelude::*;

#[derive(Debug, Clone, Widget)]
pub struct Offset {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state] offset_x: F64State,
    #[state] offset_y: F64State,
}

impl Offset {
    pub fn new(offset_x: F64State, offset_y: F64State, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Offset {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            offset_x,
            offset_y,
        })
    }
}

impl Layout for Offset {
    fn flexibility(&self) -> u32 {
        self.child.flexibility()
    }

    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        self.dimension = self.child.calculate_size(requested_size, env);
        self.dimension
    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        let mut child_position = self.child.position();

        child_position.x += *self.offset_x.value();
        child_position.y += *self.offset_y.value();

        self.child.set_position(child_position);

        self.child.position_children();
    }
}

impl CommonWidget for Offset {
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
        if self.child.flag() == Flags::PROXY {
            self.child.children()
        } else {
            WidgetIter::single(self.child.deref())
        }
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        if self.child.flag() == Flags::PROXY {
            self.child.children_mut()
        } else {
            WidgetIterMut::single(self.child.deref_mut())
        }
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::single(self.child.deref_mut())
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

impl WidgetExt for Offset {}