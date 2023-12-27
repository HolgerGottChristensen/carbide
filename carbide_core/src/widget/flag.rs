use carbide_core::widget::CommonWidget;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::flags::Flags;
use crate::widget::{Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
pub struct Flagged<C> where C: Widget {
    id: WidgetId,
    child: C,
    position: Position,
    dimension: Dimension,
    flags: Flags,
}

impl Flagged<Empty> {
    #[carbide_default_builder2]
    pub fn new<C: Widget>(child: C, flags: Flags) -> Flagged<C> {
        Flagged {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            flags,
        }
    }
}

impl<C: Widget> CommonWidget for Flagged<C> {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension, flag: self.flags);
}

impl<C: Widget> WidgetExt for Flagged<C> {}
