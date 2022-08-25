use carbide_macro::carbide_default_builder;
use crate::draw::{Dimension, Position};
use crate::CommonWidgetImpl;
use crate::flags::Flags;
use crate::widget::{Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
pub struct Flagged {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    flags: Flags,
}

impl Flagged {
    #[carbide_default_builder]
    pub fn new(child: Box<dyn Widget>, flags: Flags) -> Box<Self> {}

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
