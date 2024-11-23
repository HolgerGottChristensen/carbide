use carbide::widget::AnyWidget;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::flags::WidgetFlag;
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId, Sequence};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Proxy<W> where W: Sequence
{
    id: WidgetId,
    children: W,
    position: Position,
    dimension: Dimension,
}

impl Proxy<Empty> {
    #[carbide_default_builder2]
    pub fn new<W: Sequence>(children: W) -> Proxy<W> {
        Proxy {
            id: WidgetId::new(),
            children,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<W: Sequence> CommonWidget for Proxy<W> {
    CommonWidgetImpl!(self, id: self.id, child: self.children, position: self.position, dimension: self.dimension, flag: WidgetFlag::PROXY);
}