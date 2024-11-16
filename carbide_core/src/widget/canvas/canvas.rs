use std::fmt::{Debug, Formatter};
use carbide::widget::AnyWidget;
use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::render::Render;
use crate::render::RenderContext;
use crate::widget::{CommonWidget, PrimitiveStore, Shape, ShapeStyle, StrokeStyle, Widget, WidgetExt, WidgetId};
use crate::widget::canvas::CanvasContext;

/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Canvas<C>
where
    C: Context
{
    id: WidgetId,
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
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl<C: Context> Shape for Canvas<C> {
    fn get_triangle_store_mut(&mut self) -> &mut PrimitiveStore {
        todo!()
    }

    fn get_stroke_style(&self) -> StrokeStyle {
        todo!()
    }

    fn get_shape_style(&self) -> ShapeStyle {
        todo!()
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