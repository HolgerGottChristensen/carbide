use crate::draw::{Dimension, Position};
use crate::prelude::*;
use crate::CommonWidgetImpl;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Hidden {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
}

impl Hidden {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Hidden {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

CommonWidgetImpl!(Hidden, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl Render for Hidden {
    // Because we try to hide all children, we just stop the rendering tree.
    fn process_get_primitives(&mut self, _: &mut Vec<Primitive>, _: &mut Environment) {}
}

impl WidgetExt for Hidden {}
