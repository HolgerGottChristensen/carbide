use lyon::lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeTessellator, VertexBuffers};
use lyon::math::{point, vector, Angle, Point};
use lyon::path::{Path, Winding};
use lyon::path::builder::BorderRadii;
use lyon::tessellation::{FillRule, LineCap, StrokeAlignment, StrokeOptions};
use carbide_core::draw::{DrawShape, Position, Scalar};
use crate::stroke_vertex::StrokeVertex;
use crate::triangle::Triangle;
use lyon::tessellation::{StrokeVertex as LyonStrokeVertex};
use carbide_core::draw::path::PathInstruction;

const RADIANS_FOR_MISSING_ANGLE: f32 = 100.0;

#[derive(Debug)]
pub struct Tesselator {

}

impl Tesselator {
    pub fn new() -> Tesselator {
        Tesselator {}
    }

    fn path(&mut self, draw_shape: DrawShape) -> Path {
        let mut builder = Path::builder();

        match draw_shape {
            DrawShape::Rectangle(rect) => {
                let rect = lyon::geom::euclid::rect(
                    rect.position.x as f32,
                    rect.position.y as f32,
                    rect.dimension.width as f32,
                    rect.dimension.height as f32,
                );

                builder.add_rectangle(&rect.to_box2d(), Winding::Positive);
            }
            DrawShape::Capsule(rect) => {
                let rect = lyon::geom::euclid::rect(
                    rect.position.x as f32,
                    rect.position.y as f32,
                    rect.dimension.width as f32,
                    rect.dimension.height as f32,
                );

                builder.add_rounded_rectangle(
                    &rect.to_box2d(),
                    &BorderRadii {
                        top_left: f32::MAX,
                        top_right: f32::MAX,
                        bottom_left: f32::MAX,
                        bottom_right: f32::MAX,
                    },
                    Winding::Positive,
                );
            }
            DrawShape::RoundedRectangle(rect, corners) => {
                let rect = lyon::geom::euclid::rect(
                    rect.position.x as f32,
                    rect.position.y as f32,
                    rect.dimension.width as f32,
                    rect.dimension.height as f32,
                );

                builder.add_rounded_rectangle(
                    &rect.to_box2d(),
                    &BorderRadii {
                        top_left: corners.top_left as f32,
                        top_right: corners.top_right as f32,
                        bottom_left: corners.bottom_left as f32,
                        bottom_right: corners.bottom_right as f32,
                    },
                    Winding::Positive,
                );
            }
            DrawShape::Circle(center, radius) => {
                builder.add_circle(point(center.x as f32, center.y as f32), radius as f32, Winding::Positive);
            }
            DrawShape::Ellipse(rect) => {
                let center = rect.center();
                builder.add_ellipse(point(center.x as f32, center.y as f32), vector(rect.width() as f32 / 2.0, rect.height() as f32 / 2.0), Angle::degrees(0.0), Winding::Positive);
            }
            DrawShape::Path(path) => {
                let mut builder = Path::builder().with_svg();

                for instruction in path.instructions {
                    match instruction {
                        PathInstruction::MoveTo { to } => {
                            builder.move_to(point(to.x as f32, to.y as f32));
                        },
                        PathInstruction::Close => {
                            builder.close()
                        },
                        PathInstruction::LineTo { to } => {
                            builder.line_to(point(to.x as f32, to.y as f32));
                        }
                        PathInstruction::QuadraticBezierTo { ctrl, to } => {
                            builder.quadratic_bezier_to(point(ctrl.x as f32, ctrl.y as f32), point(to.x as f32, to.y as f32));
                        }
                        PathInstruction::CubicBezierTo { ctrl1, ctrl2, to } => {
                            builder.cubic_bezier_to(point(ctrl1.x as f32, ctrl1.y as f32), point(ctrl2.x as f32, ctrl2.y as f32), point(to.x as f32, to.y as f32));
                        }
                        PathInstruction::Arc { center, radius, start_angle, end_angle } => {}
                    }
                }

                return builder.build();
            }
            DrawShape::Single(_) => {}
            DrawShape::Multiple(_) => {}
        }

        builder.build()
    }

    pub fn fill(&mut self, draw_shape: DrawShape, options: carbide_core::draw::fill::FillOptions) -> impl Iterator<Item=Triangle<Position>> {

        let path = self.path(draw_shape);

        let mut geometry: VertexBuffers<Position, u16> = VertexBuffers::new();

        let mut tessellator = FillTessellator::new();

        let fill_rule = match options.fill_rule {
            carbide_core::draw::fill::FillRule::EvenOdd => FillRule::EvenOdd,
            carbide_core::draw::fill::FillRule::NonZero => FillRule::NonZero
        };

        let fill_options = FillOptions::default()
            .with_fill_rule(fill_rule);

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

        let triangles = Triangle::from_point_list(points);

        triangles.into_iter()
    }

    pub fn stroke(&mut self, draw_shape: DrawShape, options: carbide_core::draw::stroke::StrokeOptions) -> impl Iterator<Item=Triangle<StrokeVertex>> {
        let path = self.path(draw_shape);

        #[derive(Debug, Copy, Clone, PartialEq)]
        pub struct Vertex {
            position: Point,

            prev: Option<Point>,
            current: Point,
            next: Option<Point>,

            width: f32,
            offset: f32,
        }


        let mut geometry: VertexBuffers<Vertex, u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();

        //println!("{:?}", path);

        let stroke_width = options.stroke_width as f32;

        let stroke_alignment = match options.stroke_alignment {
            carbide_core::draw::stroke::StrokeAlignment::Center => StrokeAlignment::Center,
            carbide_core::draw::stroke::StrokeAlignment::Positive => StrokeAlignment::Positive,
            carbide_core::draw::stroke::StrokeAlignment::Negative => StrokeAlignment::Negative,
        };



        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &StrokeOptions::default()
                        .with_line_width(stroke_width)
                        .with_alignment(stroke_alignment),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: LyonStrokeVertex| {
                        //dbg!(&vertex);

                        Vertex {
                            position: vertex.position(),
                            prev: vertex.prev_position_on_path(),
                            current: vertex.position_on_path(),
                            next: vertex.next_position_on_path(),
                            width: vertex.line_width(),
                            offset: vertex.advancement(),
                        }
                    }),
                )
                .unwrap();
        }

        let triangles = geometry
            .indices
            .chunks(3)
            .map(|indices| {
                let vertex0 = geometry.vertices[indices[0] as usize];
                let vertex1 = geometry.vertices[indices[1] as usize];
                let vertex2 = geometry.vertices[indices[2] as usize];

                let first = vertex0;

                let second = if vertex1.offset != vertex0.offset { vertex1 } else { vertex2 };

                let (min, max) = if first.offset < second.offset {
                    (first, second)
                } else {
                    (second, first)
                };

                let start_angle = if let Some(prev) = min.prev {
                    let b = min.current - prev;
                    - b.angle_from_x_axis() + Angle::frac_pi_2()
                } else {
                    Angle::radians(RADIANS_FOR_MISSING_ANGLE)
                };

                let end_angle = if let Some(next) = max.next {
                    let b = max.current - next;
                    - b.angle_from_x_axis() - Angle::frac_pi_2()
                } else {
                    Angle::radians(RADIANS_FOR_MISSING_ANGLE)
                };

                Triangle([
                    StrokeVertex {
                        position: vertex0.position,
                        start: min.current,
                        end: max.current,
                        start_angle,
                        end_angle,
                        width: vertex0.width,
                        offset: min.offset,
                    },
                    StrokeVertex {
                        position: vertex1.position,
                        start: min.current,
                        end: max.current,
                        start_angle,
                        end_angle,
                        width: vertex1.width,
                        offset: min.offset,
                    },
                    StrokeVertex {
                        position: vertex2.position,
                        start: min.current,
                        end: max.current,
                        start_angle,
                        end_angle,
                        width: vertex2.width,
                        offset: min.offset,
                    },
                ])
            }).collect::<Vec<_>>();

        triangles.into_iter()
    }
}