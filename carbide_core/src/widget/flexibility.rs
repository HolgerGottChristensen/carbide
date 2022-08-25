use carbide_macro::carbide_default_builder;
use crate::draw::{Dimension, Position};
use crate::CommonWidgetImpl;
use crate::widget::{Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
pub struct Flexibility {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    flexibility: u32,
}

impl Flexibility {
    #[carbide_default_builder]
    pub fn new(child: Box<dyn Widget>, flexibility: u32) -> Box<Self> {}

    pub fn new(child: Box<dyn Widget>, flexibility: u32) -> Box<Self> {
        Box::new(Flexibility {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            flexibility,
        })
    }
}

CommonWidgetImpl!(Flexibility, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: self.flexibility);

impl WidgetExt for Flexibility {}
