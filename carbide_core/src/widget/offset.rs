use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::CommonWidgetImpl;

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

CommonWidgetImpl!(Offset, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl WidgetExt for Offset {}
