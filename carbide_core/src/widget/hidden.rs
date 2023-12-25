use carbide_core::render::RenderContext;
use carbide_core::widget::CommonWidget;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::render::Render;
use crate::widget::{Empty, Widget, WidgetExt, WidgetId};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Hidden<W> where W: Widget {
    id: WidgetId,
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
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl<W: Widget> Render for Hidden<W> {
    // Because we try to hide all children, we just stop the rendering tree.
    fn render(&mut self, _: &mut RenderContext, _: &mut Environment) {}
}

impl<W: Widget> WidgetExt for Hidden<W> {}
