use carbide_core::render::RenderContext;
use carbide_core::widget::CommonWidget;
use carbide_macro::carbide_default_builder;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::environment::Environment;
use crate::render::Render;
use crate::widget::{AnyWidget, Widget, WidgetExt, WidgetId};

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Hidden {
    id: WidgetId,
    child: Box<dyn AnyWidget>,
    position: Position,
    dimension: Dimension,
}

impl Hidden {
    #[carbide_default_builder]
    pub fn new(child: Box<dyn AnyWidget>) -> Box<Self> {}

    pub fn new(child: Box<dyn AnyWidget>) -> Box<Self> {
        Box::new(Hidden {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
        })
    }
}

impl CommonWidget for Hidden {
    CommonWidgetImpl!(self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);
}

impl Render for Hidden {
    // Because we try to hide all children, we just stop the rendering tree.
    fn render(&mut self, _: &mut RenderContext, _: &mut Environment) {}
}

impl WidgetExt for Hidden {}
