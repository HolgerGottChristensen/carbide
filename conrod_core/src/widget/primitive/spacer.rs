use crate::prelude::*;
use crate::widget::types::spacer_direction::SpacerDirection;


#[derive(Clone, Debug, Widget)]
pub struct Spacer {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    space: SpacerDirection,
}

impl<GS: GlobalState> WidgetExt<GS> for Spacer {}

impl Spacer {
    pub fn new(space: SpacerDirection) -> Box<Self> {
        Box::new(Spacer {
            id: Uuid::new_v4(),
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            space
        })
    }
}

impl<S: GlobalState> Layout<S> for Spacer {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, _env: &Environment<S>) -> Dimensions {
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

    fn position_children(&mut self) {

    }
}

impl<S: GlobalState> CommonWidget<S> for Spacer {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_flag(&self) -> Flags {
        Flags::SPACER
    }

    fn get_children(&self) -> WidgetIter<S> {
        WidgetIter::Empty
    }

    fn get_children_mut(&mut self) -> WidgetIterMut<S> {
        WidgetIterMut::Empty
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut<S> {
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

impl<S: GlobalState> Render<S> for Spacer {
    fn get_primitives(&mut self, _fonts: &text::font::Map) -> Vec<Primitive> {
        let mut prims = vec![];
        prims.extend(Rectangle::<S>::debug_outline(Rect::new(self.position, self.dimension), 1.0));
        return prims;
    }
}



