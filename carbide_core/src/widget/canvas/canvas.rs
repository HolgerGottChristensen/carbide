use lyon::algorithms::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers};

use crate::color::Rgba;
use crate::draw::shape::triangle::Triangle;
use crate::prelude::*;
use crate::render::PrimitiveKind;
use crate::widget::canvas::{Context, ShapeStyleWithOptions};
use crate::widget::Rectangle;

/// A basic, non-interactive rectangle shape widget.
#[derive(Debug, Clone, Widget)]
pub struct Canvas {
    id: Uuid,
    position: Point,
    dimension: Dimensions,
    #[state] color: ColorState,
    //prim_store: Vec<Primitive>,
    context: fn(OldRect, Context) -> Context,
}

impl Canvas {
    pub fn initialize(context: fn(OldRect, Context) -> Context) -> Box<Self> {
        Box::new(Canvas {
            id: Uuid::new_v4(),
            position: [0.0, 0.0],
            dimension: [100.0, 100.0],
            color: EnvironmentColor::Accent.into(),
            //prim_store: vec![],
            context,
        })
    }

    pub fn get_stroke_prim(&self, path: Path, stroke_options: StrokeOptions, color: Color) -> Primitive {
        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();

        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &stroke_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    let point = vertex.position().to_array();
                    [point[0] as Scalar, point[1] as Scalar]
                }),
            ).unwrap();
        }

        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Point> = point_iter.collect();

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(color), triangles: Triangle::from_point_list(points) },
            rect: OldRect::new(self.position, self.dimension),
        }
    }

    pub fn get_fill_prim(&self, path: Path, fill_options: FillOptions, color: Color) -> Primitive {
        let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();

        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &fill_options,
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    let point = vertex.position().to_array();
                    [point[0] as Scalar, point[1] as Scalar]
                }),
            ).unwrap();
        }

        let point_iter = geometry.indices.iter().map(|index| geometry.vertices[*index as usize]);

        let points: Vec<Point> = point_iter.collect();

        Primitive {
            kind: PrimitiveKind::TrianglesSingleColor { color: Rgba::from(color), triangles: Triangle::from_point_list(points) },
            rect: OldRect::new(self.position, self.dimension),
        }
    }
}

impl CommonWidget for Canvas {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    fn get_flag(&self) -> Flags {
        Flags::EMPTY
    }

    fn get_children(&self) -> WidgetIter {
        WidgetIter::Empty
    }

    fn get_children_mut(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_proxied_children(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_proxied_children_rev(&mut self) -> WidgetIterMut {
        WidgetIterMut::Empty
    }

    fn get_position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Dimensions) {
        self.position = position;
    }

    fn get_dimension(&self) -> Dimensions {
        self.dimension
    }

    fn set_dimension(&mut self, dimensions: Dimensions) {
        self.dimension = dimensions
    }
}

impl Render for Canvas {
    fn get_primitives(&mut self, env: &mut Environment) -> Vec<Primitive> {
        let context = Context::new();

        let rectangle = OldRect::new(self.get_position(), self.get_dimension());
        let context = (self.context)(rectangle, context);

        let paths = context.to_paths(self.get_position());

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

        prims.extend(Rectangle::debug_outline(OldRect::new(self.position, self.dimension), 1.0));

        return prims;
    }
}

impl WidgetExt for Canvas {}

impl Layout for Canvas {
    fn flexibility(&self) -> u32 {
        0
    }

    fn calculate_size(&mut self, requested_size: Dimensions, env: &mut Environment) -> Dimensions {
        self.dimension = requested_size;
        requested_size
    }

    fn position_children(&mut self) {}
}