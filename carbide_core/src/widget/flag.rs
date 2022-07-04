use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
pub struct Flagged {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    flags: Flags,
}

impl Flagged {
    pub fn new(child: Box<dyn Widget>, flags: Flags) -> Box<Self> {
        Box::new(Flagged {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            flags,
        })
    }
}

CommonWidgetImpl!(Flagged, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: self.flags);

impl WidgetExt for Flagged {}
