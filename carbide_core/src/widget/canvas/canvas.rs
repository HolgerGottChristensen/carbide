use std::fmt::{Debug, Formatter};

use lyon::algorithms::path::Path;
use lyon::math::point;
use lyon::path::Side;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect, Scalar};
use crate::draw::shape::triangle::Triangle;
use crate::environment::Environment;
use crate::render::Render;
use crate::render::RenderContext;
use crate::widget::{CommonWidget, PrimitiveStore, Shape, ShapeStyle, StrokeStyle, Widget, WidgetExt, WidgetId};
use crate::widget::canvas::{CanvasContext};

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

    fn triangles(&mut self, env: &mut Environment) -> Vec<Triangle<Position>> {
        /*let mut context = CanvasContext::new(self.position, self.dimension);

        let rectangle = Rect::new(self.position(), self.dimension());

        self.context.call(rectangle, &mut context, env);

        let paths = context.to_paths(self.position(), env);
        let mut triangles = vec![];

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, _) => {
                    triangles.extend(context.get_fill_geometry(path, fill_options));
                }
                ShapeStyleWithOptions::Stroke(stroke_options, _, _) => {
                    triangles.extend(context.get_stroke_geometry(path, stroke_options).iter().map(|a| Triangle([
                        a.0[0].0,
                        a.0[1].0,
                        a.0[2].0
                    ])));
                }
                ShapeStyleWithOptions::Clip => {}
                ShapeStyleWithOptions::UnClip => {}
            }
        }

        triangles*/

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

impl<C: Context> WidgetExt for Canvas<C> {}
