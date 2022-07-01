use std::fmt::{Debug, Formatter};
use lyon::algorithms::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

use crate::color::Rgba;
use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::draw::shape::triangle::Triangle;
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::widget::canvas::{Context, ShapeStyleWithOptions};
use crate::widget::canvas::canvas::Contexts::{NoState, WithState};

/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Widget)]
#[carbide_exclude(Render)]
pub struct Canvas<T> where T: StateContract {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    #[state]
    color: ColorState,
    //prim_store: Vec<Primitive>,
    context: Contexts<T>,
    #[state] state: TState<T>,
}

#[derive(Clone)]
enum Contexts<T> where T: StateContract {
    WithState(fn(&mut TState<T>, Rect, Context, &mut Environment) -> Context),
    NoState(fn(Rect, Context, &mut Environment) -> Context)
}

impl Canvas<()> {
    pub fn new(context: fn(Rect, Context, &mut Environment) -> Context) -> Box<Canvas<()>> {
        Box::new(Canvas {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            color: EnvironmentColor::Accent.into(),
            //prim_store: vec![],
            context: NoState(context),
            state: ValueState::new(())
        })
    }
}

impl<T: StateContract> Canvas<T> {
    pub fn new_with_state(state: impl Into<TState<T>>, context: fn(&mut TState<T>, Rect, Context, &mut Environment) -> Context) -> Box<Canvas<T>> {
        Box::new(Canvas {
            id: WidgetId::new(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            color: EnvironmentColor::Accent.into(),
            //prim_store: vec![],
            context: WithState(context),
            state: state.into()
        })
    }

    pub fn get_stroke_prim(
        &self,
        path: Path,
        stroke_options: StrokeOptions,
        color: Color,
    ) -> Primitive {
        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();

        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &stroke_options,
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
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

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor {
                color,
                triangles: Triangle::from_point_list(points),
            },
            bounding_box: Rect::new(self.position, self.dimension),
        }
    }

    pub fn get_fill_prim(&self, path: Path, fill_options: FillOptions, color: Color) -> Primitive {
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

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor {
                color,
                triangles: Triangle::from_point_list(points),
            },
            bounding_box: Rect::new(self.position, self.dimension),
        }
    }
}

impl<T: StateContract> CommonWidget for Canvas<T> {
    fn id(&self) -> carbide_core::widget::WidgetId {
        (self.id)
    }


    fn children(&self) -> carbide_core::widget::WidgetIter {
        carbide_core::widget::WidgetIter::Empty
    }

    fn children_mut(&mut self) -> carbide_core::widget::WidgetIterMut {
        carbide_core::widget::WidgetIterMut::Empty
    }

    fn children_direct(&mut self) -> carbide_core::widget::WidgetIterMut {
        carbide_core::widget::WidgetIterMut::Empty
    }

    fn children_direct_rev(&mut self) -> carbide_core::widget::WidgetIterMut {
        carbide_core::widget::WidgetIterMut::Empty
    }

    fn position(&self) -> carbide_core::draw::Position {
        (self.position)
    }

    fn set_position(&mut self, position: carbide_core::draw::Position) {
        (self.position) = position;
    }

    fn dimension(&self) -> carbide_core::draw::Dimension {
        (self.dimension)
    }

    fn set_dimension(&mut self, dimension: carbide_core::draw::Dimension) {
        (self.dimension) = dimension
    }
}

impl<T: StateContract> Shape for Canvas<T> {
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
        let context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());

        let context = match self.context {
            WithState(c) => {
                c(&mut self.state, rectangle, context, env)
            }
            NoState(c) => {
                c(rectangle, context, env)
            }
        };

        let paths = context.to_paths(self.position());
        let mut prims = vec![];

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, mut color) => {
                    color.sync(env);
                    prims.push(self.get_fill_prim(path, fill_options, *color.value()));
                    //color.release_state(env);
                }
                ShapeStyleWithOptions::Stroke(stroke_options, mut color) => {
                    color.sync(env);
                    prims.push(self.get_stroke_prim(path, stroke_options, *color.value()));
                    //color.release_state(env);
                }
            }
        }

        let mut res_triangle_list = vec![];

        for prim in prims {
            match prim.kind {
                PrimitiveKind::TrianglesSingleColor { triangles, .. } => {
                    res_triangle_list.extend(triangles);
                }
                _ => (),
            }
        }

        res_triangle_list
    }
}

impl<T: StateContract> Render for Canvas<T> {
    fn get_primitives(&mut self, primitives: &mut Vec<Primitive>, env: &mut Environment) {
        let context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());
        let context = match self.context {
            WithState(c) => {
                c(&mut self.state, rectangle, context, env)
            }
            NoState(c) => {
                c(rectangle, context, env)
            }
        };

        let paths = context.to_paths(self.position());

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, mut color) => {
                    color.sync(env);
                    primitives.push(self.get_fill_prim(path, fill_options, *color.value()));
                    //color.release_state(env);
                }
                ShapeStyleWithOptions::Stroke(stroke_options, mut color) => {
                    color.sync(env);
                    primitives.push(self.get_stroke_prim(path, stroke_options, *color.value()));
                    //color.release_state(env);
                }
            }
        }
    }
}

impl<T: StateContract> Debug for Canvas<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Canvas")
            .finish()
    }
}

impl<T: StateContract> WidgetExt for Canvas<T> {}
