use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::prelude::*;
use crate::render::PrimitiveKind;

#[derive(Debug, Clone, Widget)]
pub struct Flexibility {
    id: Uuid,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    flexibility: u32,
}

impl Flexibility {
    pub fn new(child: Box<dyn Widget>, flexibility: u32) -> Box<Self> {
        Box::new(Flexibility {
            id: Uuid::new_v4(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            flexibility
        })
    }
}

CommonWidgetImpl!(Flexibility, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: self.flexibility);

impl WidgetExt for Flexibility {}
