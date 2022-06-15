use carbide_core::CommonWidgetImpl;

use crate::draw::{Dimension, Position};
use crate::prelude::*;

#[derive(Clone, Debug, Widget)]
pub struct Empty {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
}

impl Empty {
    pub fn new() -> Box<Self> {
        Box::new(Empty {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

CommonWidgetImpl!(Empty, self, id: self.id, position: self.position, dimension: self.dimension, flag: Flags::IGNORE);

impl WidgetExt for Empty {}
