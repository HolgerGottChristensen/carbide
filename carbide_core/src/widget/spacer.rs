use crate::prelude::*;
use crate::render::ChildRender;
use crate::widget::types::SpacerDirection;

#[derive(Clone, Debug, Widget)]
pub struct Spacer {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    space: SpacerDirection,
}

impl Spacer {
    pub fn new(space: SpacerDirection) -> Box<Self> {
        Box::new(Spacer {
            id: Uuid::new_v4(),
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            space,
        })
    }
}

impl Layout for Spacer {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, _: &mut Environment) -> Dimensions {
        match self.space {
            SpacerDirection::Vertical => {
                self.dimension = [0.0, requested_size[1]];
            }
            SpacerDirection::Horizontal => {
                self.dimension = [requested_size[0], 0.0];
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

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl ChildRender for Spacer {}

impl WidgetExt for Spacer {}



