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
use crate::widget::canvas::{Context, ShapeStyleWithOptions};

/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Canvas<C>
where
    C: CanvasContext
{
    id: WidgetId,
    position: Position,
    dimension: Dimension,

    context: C,
}

pub trait CanvasContext: Clone + 'static {
    fn call(&mut self, area: Rect, context: &mut Context, env: &mut Environment);
}

impl<T> CanvasContext for T where T: Fn(Rect, &mut Context, &mut Environment) + Clone + 'static {
    fn call(&mut self, area: Rect, context: &mut Context, env: &mut Environment) {
        self(area, context, env)
    }
}

type DefaultCanvasContext = fn(Rect, &mut Context, &mut Environment);

impl Canvas<DefaultCanvasContext> {

    #[carbide_default_builder2]
    pub fn new<C: CanvasContext>(context: C) -> Canvas<C> {
        Canvas {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),

            context,
        }
    }
}

impl<C: CanvasContext> Canvas<C> {
    pub fn get_fill_geometry(&self, path: Path, fill_options: FillOptions) -> Vec<Triangle<Position>> {
        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &fill_options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        let point = vertex.position().to_array();
                        Position::new(point[0] as Scalar, point[1] as Scalar)
                    }),
                )
                .unwrap();
        }

        let point_iter = geometry
            .indices
            .iter()
            .map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Position> = point_iter.collect();

        Triangle::from_point_list(points)
    }

    pub fn get_stroke_geometry(
        &self,
        path: Path,
        stroke_options: StrokeOptions,
    ) -> Vec<Triangle<(Position, (Position, Position, f32, f32))>> {
        let mut geometry: VertexBuffers<(Position, f32), u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();

        //println!("{:?}", path);

        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &stroke_options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                        /*dbg!(
                            &vertex.position(),
                            &vertex.advancement(),
                            &vertex.source(),
                            &vertex.normal(),
                        );*/
                        let point = vertex.position();

                        (Position::new(point.x as Scalar, point.y as Scalar), vertex.line_width())
                    }),
                )
                .unwrap();
        }

        let point_iter = geometry
            .indices
            .iter()
            .enumerate()
            .map(|(e, index)| {
                //let dir = geometry.points[e / 3];
                let dir = (point(0.0, 0.0), point(400.0, 400.0), 0.0);
                (geometry.vertices[*index as usize].0, (Position::new(dir.0.x as f64, dir.0.y as f64), Position::new(dir.1.x as f64, dir.1.y as f64), dir.2, geometry.vertices[*index as usize].1))
            });

        let points: Vec<_> = point_iter.collect();

        Triangle::from_point_list(points)
    }
}

impl<C: CanvasContext> CommonWidget for Canvas<C> {
    CommonWidgetImpl!(self, id: self.id, child: (), position: self.position, dimension: self.dimension);
}

impl<C: CanvasContext> Shape for Canvas<C> {
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
        let mut context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());

        self.context.call(rectangle, &mut context, env);

        let paths = context.to_paths(self.position(), env);
        let mut triangles = vec![];

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, _) => {
                    triangles.extend(self.get_fill_geometry(path, fill_options));
                }
                ShapeStyleWithOptions::Stroke(stroke_options, _, _) => {
                    triangles.extend(self.get_stroke_geometry(path, stroke_options).iter().map(|a| Triangle([
                        a.0[0].0,
                        a.0[1].0,
                        a.0[2].0
                    ])));
                }
            }
        }

        triangles
    }
}

impl<C: CanvasContext> Render for Canvas<C> {
    fn render(&mut self, render_context: &mut RenderContext) {
        let mut context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());
        self.context.call(rectangle, &mut context, render_context.env);

        let paths = context.to_paths(self.position(), render_context.env);

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, style) => {
                    render_context.style(style.convert(self.position, self.dimension), |this| {
                        this.geometry(&self.get_fill_geometry(path, fill_options))
                    })
                }
                ShapeStyleWithOptions::Stroke(stroke_options, style, dashes) => {
                    render_context.style(style.convert(self.position, self.dimension), |render_context| {
                        render_context.stroke_dash_pattern(dashes, |render_context| {
                            render_context.stroke(&self.get_stroke_geometry(path, stroke_options))
                        })
                    })
                }
            }
        }
    }
}

impl<C: CanvasContext> Debug for Canvas<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Canvas").finish()
    }
}

impl<C: CanvasContext> WidgetExt for Canvas<C> {}
