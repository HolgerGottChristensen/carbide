use carbide_core::CommonWidgetImpl;
use carbide_macro::carbide_default_builder;

use crate::draw::{Dimension, Position};
use crate::Scene;
use crate::flags::Flags;
use crate::widget::{Widget, WidgetExt, WidgetId};

#[derive(Clone, Debug, Widget)]
pub struct Empty {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
}

impl Empty {
    #[carbide_default_builder]
    pub fn new() -> Box<Self> {}

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

impl Scene for Empty {
    fn request_redraw(&self) {
        // Empty request redraw, because no redrawing is necessary.
    }
}
