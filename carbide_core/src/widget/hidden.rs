use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::render::{Render, RenderContext};
use crate::widget::{Empty, Widget, WidgetExt, WidgetId, CommonWidget};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Hidden<W> where W: Widget {
    #[id] id: WidgetId,
    child: W,
    position: Position,
    dimension: Dimension,
}

impl Hidden<Empty> {
    #[carbide_default_builder2]
    pub fn new<W: Widget>(child: W) -> Hidden<W> {
        Hidden {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        }
    }
}

impl<W: Widget> CommonWidget for Hidden<W> {
    CommonWidgetImpl!(self, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: Widget> Render for Hidden<W> {
    // Because we try to hide all children, we just stop the rendering tree.
    fn render(&mut self, _context: &mut RenderContext) {}
}