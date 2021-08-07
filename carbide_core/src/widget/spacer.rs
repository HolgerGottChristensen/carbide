use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::render::ChildRender;
use crate::widget::types::SpacerDirection;

#[derive(Clone, Debug, Widget)]
pub struct Spacer {
    id: Uuid,
    position: Position,
    dimension: Dimension,
    space: SpacerDirection,
}

impl Spacer {
    pub fn new(space: SpacerDirection) -> Box<Self> {
        Box::new(Spacer {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            space,
        })
    }
}

impl Layout for Spacer {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        match self.space {
            SpacerDirection::Vertical => {
                self.dimension = Dimension::new(0.0, requested_size.height);
            }
            SpacerDirection::Horizontal => {
                self.dimension = Dimension::new(requested_size.width, 0.0);
            }
            SpacerDirection::Both => {
                self.dimension = requested_size;
            }
        }

        self.dimension
    }

    fn position_children(&mut self) {}
}

impl CommonWidget for Spacer {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::SPACER
    }

    fn get_children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimension) {
        self.dimension = dimensions
    }
}

impl ChildRender for Spacer {}

impl WidgetExt for Spacer {}



