use carbide_core::widget::{CommonWidget, Empty};
use carbide_macro::{carbide_default_builder2};

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::widget::{Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
pub struct Flexibility<C> where C: Widget + Clone {
    id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    flexibility: u32,
}

impl Flexibility<Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget + Clone>(child: C, flexibility: u32) -> Box<Flexibility<C>> {
        Box::new(Flexibility {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            flexibility,
        })
    }
}

impl<C: Widget + Clone> CommonWidget for Flexibility<C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flexibility: self.flexibility);
}

impl<C: Widget + Clone> WidgetExt for Flexibility<C> {}
