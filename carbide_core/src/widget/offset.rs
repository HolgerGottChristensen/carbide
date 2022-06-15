use crate::draw::{Dimension, Position};
use crate::prelude::*;

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Offset {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state]
    offset_x: F64State,
    #[state]
    offset_y: F64State,
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
    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.positioner();
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
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
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

impl WidgetExt for Offset {}
