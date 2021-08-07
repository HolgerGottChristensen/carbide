use lyon::algorithms::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers};

use crate::color::Rgba;
use crate::draw::{Dimension, Position, Rect};
use crate::draw::shape::triangle::Triangle;
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::widget::canvas::{Context, ShapeStyleWithOptions};
use crate::widget::Rectangle;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Canvas {
    id: Uuid,
    position: Position,
    dimension: Dimension,
    #[state] color: ColorState,
    //prim_store: Vec<Primitive>,
    context: fn(Rect, Context) -> Context,
}

impl Canvas {
    pub fn initialize(context: fn(Rect, Context) -> Context) -> Box<Self> {
        Box::new(Canvas {
            id: Uuid::new_v4(),
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(100.0, 100.0),
            color: EnvironmentColor::Accent.into(),
            //prim_store: vec![],
            context,
        })
    }

    pub fn get_stroke_prim(&self, path: Path, stroke_options: StrokeOptions, color: Color) -> Primitive {
        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();

        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &stroke_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    let point = vertex.position().to_array();
                    Position::new(point[0] as Scalar, point[1] as Scalar)
                }),
            ).unwrap();
        }

        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Position> = point_iter.collect();

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(color), triangles: Triangle::from_point_list(points) },
            rect: Rect::new(self.position, self.dimension),
        }
    }

    pub fn get_fill_prim(&self, path: Path, fill_options: FillOptions, color: Color) -> Primitive {
        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &fill_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    let point = vertex.position().to_array();
                    Position::new(point[0] as Scalar, point[1] as Scalar)
                }),
            ).unwrap();
        }

        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Position> = point_iter.collect();

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(color), triangles: Triangle::from_point_list(points) },
            rect: Rect::new(self.position, self.dimension),
        }
    }
}

impl CommonWidget for Canvas {
    fn id(&self) -> Id {
        self.id
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn position(&self) -> Position {
        self.position
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    fn dimension(&self) -> Dimension {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimension) {
        self.dimension = dimensions
    }
}

impl Render for Canvas {
    fn get_primitives(&mut self, env: &mut Environment) -> Vec<Primitive> {
        let context = Context::new();

        let rectangle = Rect::new(self.position(), self.dimension());
        let context = (self.context)(rectangle, context);

        let paths = context.to_paths(self.position());

        let mut prims = vec![];

        for (path, options) in paths {
            match options {
                ShapeStyleWithOptions::Fill(fill_options, mut color) => {
                    color.capture_state(env);
                    prims.push(self.get_fill_prim(path, fill_options, *color.value()));
                    color.release_state(env);
                }
                ShapeStyleWithOptions::Stroke(stroke_options, mut color) => {
                    color.capture_state(env);
                    prims.push(self.get_stroke_prim(path, stroke_options, *color.value()));
                    color.release_state(env);
                }
            }
        }

        prims.extend(Rectangle::debug_outline(Rect::new(self.position, self.dimension), 1.0));

        return prims;
    }
}

impl WidgetExt for Canvas {}

impl Layout for Canvas {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimension, _: &mut Environment) -> Dimension {
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {}
}