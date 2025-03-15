use std::fmt::{Debug, Formatter};
use carbide::draw::DrawOptions;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, DrawShape, Position};
use crate::render::Render;
use crate::render::RenderContext;
use crate::widget::{CommonWidget, AnyShape, Widget, WidgetId, ShapeStyle};
use crate::widget::canvas::CanvasContext;

/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Canvas<C>
where
    C: Context
{
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    context: C,
}

pub trait Context: Clone + 'static {
    fn call(&mut self, context: &mut CanvasContext);
}

impl<T> Context for T where T: Fn(&mut CanvasContext) + Clone + 'static {
    fn call(&mut self, context: &mut CanvasContext) {
        self(context)
    }
}

type DefaultCanvasContext = fn(&mut CanvasContext);

impl Canvas<DefaultCanvasContext> {
    #[carbide_default_builder2]
    pub fn new<C: Context>(context: C) -> Canvas<C> {
        Canvas {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),

            context,
        }
    }
}

impl<C: Context> CommonWidget for Canvas<C> {
    CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
}

impl<C: Context> AnyShape for Canvas<C> {
    fn cache_key(&self) -> Option<WidgetId> {
        todo!()
    }

    fn description(&self) -> DrawShape {
        DrawShape::Rectangle(self.bounding_box())
    }
}

impl<C: Context> Render for Canvas<C> {
    fn render(&mut self, render_context: &mut RenderContext) {
        let mut context = CanvasContext::new(self.position, self.dimension, render_context);

        self.context.call(&mut context);
    }
}

impl<C: Context> Debug for Canvas<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Canvas").finish()
    }
}