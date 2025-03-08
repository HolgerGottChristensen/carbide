use crate::animation::AnimationManager;
use crate::draw::path::PathBuilder;
use crate::draw::{Color, DrawShape};
use crate::draw::{Alignment, Angle, Dimension, Position, Scalar};
use crate::mouse_position::MousePositionEnvironmentExt;
use crate::render::{RenderContext, Style};
use crate::state::{IntoReadState, ReadState, StateSync};
use crate::text::{FontStyle, FontWeight, TextDecoration, TextId, TextStyle};
use crate::widget::{AnyShape, ShapeStyle};
use carbide::environment::Environment;
use carbide::widget::WidgetId;
use std::fmt::{Debug, Formatter};
use carbide::draw::DrawOptions;
use crate::draw::stroke::{StrokeCap, StrokeDashCap, StrokeDashMode, StrokeDashPattern, StrokeJoin, StrokeOptions};
use crate::text::text_wrap::Wrap;

pub struct CanvasContext<'a, 'b, 'c: 'b> {
    render_context: &'a mut RenderContext<'b, 'c>,
    current_state: ContextState,
    state_stack: Vec<ContextState>,
    position: Position,
    dimension: Dimension,
    path_builder: PathBuilder,
}

#[derive(Debug, Clone)]
pub struct ContextState {
    stroke_color: Style,
    fill_color: Style,
    cap_style: StrokeCap,
    join_style: StrokeJoin,
    line_width: Scalar,
    dash_pattern: Option<Vec<f64>>,
    dash_offset: Scalar,
    dash_start_cap: StrokeDashCap,
    dash_end_cap: StrokeDashCap,
    dash_mode: StrokeDashMode,
    miter_limit: Scalar,
    text_alignment: Alignment,
    clip_count: u32,
}

impl<'a, 'b, 'c: 'b> CanvasContext<'a, 'b, 'c> {
    pub fn new(position: Position, dimension: Dimension, render_context: &'a mut RenderContext<'b, 'c>) -> CanvasContext<'a, 'b, 'c> {
        CanvasContext {
            render_context,
            current_state: ContextState {
                stroke_color: Style::Color(Color::Rgba(0.0, 0.0, 0.0, 1.0)),
                fill_color: Style::Color(Color::Rgba(0.0, 0.0, 0.0, 1.0)),
                cap_style: StrokeCap::Round,
                join_style: StrokeJoin::Round,
                line_width: 2.0,
                dash_pattern: None,
                dash_offset: 0.0,
                dash_start_cap: StrokeDashCap::None,
                dash_end_cap: StrokeDashCap::None,
                dash_mode: StrokeDashMode::Pretty,
                miter_limit: 4.0, //StrokeOptions::DEFAULT_MITER_LIMIT,
                text_alignment: Alignment::TopLeading,
                clip_count: 0,
            },
            state_stack: vec![],
            position,
            dimension,
            path_builder: PathBuilder::new(),
        }
    }

    pub fn request_animation_frame(&mut self) {
        if let Some(manager) = self.render_context.env.get_mut::<AnimationManager>() {
            manager.request_animation_frame();
        }
    }

    pub fn render_context(&mut self) -> &mut RenderContext<'b, 'c> {
        self.render_context
    }

    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    pub fn width(&self) -> Scalar {
        self.dimension.width
    }

    pub fn height(&self) -> Scalar {
        self.dimension.height
    }

    pub fn mouse_position(&self) -> Position {
        let pos = self.render_context.env.mouse_position();
        pos - self.position
    }

    pub fn env(&mut self) -> &mut Environment<'c> {
        self.render_context.env
    }

    pub fn set_line_width(&mut self, width: f64) {
        self.current_state.line_width = width;
    }

    pub fn set_dash_pattern(&mut self, pattern: Option<Vec<f64>>) {
        self.current_state.dash_pattern = pattern;
    }

    pub fn set_dash_offset(&mut self, offset: f64) {
        self.current_state.dash_offset = offset;
    }

    pub fn set_dash_cap(&mut self, cap: StrokeDashCap) {
        self.set_dash_start_cap(cap);
        self.set_dash_end_cap(cap);
    }

    pub fn set_dash_start_cap(&mut self, cap: StrokeDashCap) {
        self.current_state.dash_start_cap = cap;
    }

    pub fn set_dash_end_cap(&mut self, cap: StrokeDashCap) {
        self.current_state.dash_end_cap = cap;
    }

    pub fn set_dash_mode(&mut self, mode: StrokeDashMode) {
        self.current_state.dash_mode = mode;
    }

    pub fn set_line_join(&mut self, join: StrokeJoin) {
        self.current_state.join_style = join;
    }

    pub fn set_line_cap(&mut self, cap: StrokeCap) {
        self.current_state.cap_style = cap;
    }

    pub fn set_miter_limit(&mut self, limit: Scalar) {
        self.current_state.miter_limit = limit;
    }

    pub fn set_fill_style<C: IntoReadState<Style>>(&mut self, style: C) {
        let mut read_state = style.into_read_state();
        read_state.sync(self.render_context.env);
        self.current_state.fill_color = read_state.value().clone();
    }

    pub fn set_stroke_style<C: IntoReadState<Style>>(&mut self, style: C) {
        let mut read_state = style.into_read_state();
        read_state.sync(self.render_context.env);
        self.current_state.stroke_color = read_state.value().clone();
    }

    pub fn rect(&mut self, _x: f64, _y: f64, _width: f64, _height: f64) {
        todo!()
    }

    /// x, y is the center of the circle
    pub fn circle(&mut self, x: f64, y: f64, diameter: f64) {
        self.move_to(x, y + diameter / 2.0);
        self.arc(
            Position::new(x, y),
            diameter / 2.0,
            Angle::Degrees(0.0),
            Angle::Degrees(360.0),
        );
        self.move_to(0.0, 0.0);
    }

    pub fn clear_rect(&mut self) {
        todo!()
    }

    pub fn fill(&mut self) {
        #[derive(Clone, Debug)]
        struct FillShape {
            path: carbide_core::draw::path::Path,
        }

        impl AnyShape for FillShape {
            fn cache_key(&self) -> Option<WidgetId> {
                todo!()
            }

            fn description(&self) -> DrawShape {
                DrawShape::Path(self.path.clone())
            }

            fn options(&self) -> DrawOptions {
                todo!()
            }
        }

        let path = self.path_builder.path().clone();

        self.render_context.style(self.current_state.fill_color.convert(self.position, self.dimension), |ctx| {
            ctx.shape(&FillShape { path }, ShapeStyle::Fill)
        });
    }

    pub fn stroke(&mut self) {
        #[derive(Clone, Debug)]
        struct StrokeShape {
            path: carbide_core::draw::path::Path,
        }

        impl AnyShape for StrokeShape {
            fn cache_key(&self) -> Option<WidgetId> {
                todo!()
            }

            fn description(&self) -> DrawShape {
                DrawShape::Path(self.path.clone())
            }

            fn options(&self) -> DrawOptions {
                todo!()
            }
        }

        let stroke_options = StrokeOptions::default()
            .with_stroke_cap(self.current_state.cap_style)
            .with_stroke_width(self.current_state.line_width)
            .with_miter_limit(self.current_state.miter_limit)
            .with_stroke_join(self.current_state.join_style);

        let dashes = self.current_state.dash_pattern.as_ref().map(|pattern: &Vec<f64>| {
            StrokeDashPattern {
                pattern: pattern.clone(),
                offset: self.current_state.dash_offset,
                start_cap: self.current_state.dash_start_cap,
                end_cap: self.current_state.dash_end_cap,
                dash_type: self.current_state.dash_mode,
            }
        });

        let path = self.path_builder.path().clone();

        self.render_context.style(self.current_state.stroke_color.convert(self.position, self.dimension), |ctx| {
            ctx.stroke_dash_pattern(dashes, |ctx| {
                ctx.shape(&StrokeShape { path }, stroke_options)
            })
        });
    }

    pub fn begin_path(&mut self) {
        self.path_builder = PathBuilder::new();
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        let position = Position::new(x, y) + self.position;
        self.path_builder.move_to(position);
    }

    pub fn close_path(&mut self) {
        self.path_builder.close();
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        let position = Position::new(x, y) + self.position;
        self.path_builder.line_to(position);
    }

    pub fn add_lines(&mut self, lines: impl IntoIterator<Item=Position>) {
        for line in lines {
            let position = line + self.position;
            self.path_builder.line_to(position);
        }
    }

    pub fn clip(&mut self) {
        /*let path = self.path_builder.clone().build();
        let fill_options = FillOptions::default();
        let geometry = self.get_fill_geometry(path, fill_options);
        self.render_context.render.stencil(&geometry);
        self.current_state.clip_count += 1;*/
        todo!()
    }

    pub fn save(&mut self) {
        self.state_stack.push(self.current_state.clone());
    }

    pub fn restore(&mut self) {
        let current_clip = self.current_state.clip_count;

        if let Some(state) = self.state_stack.pop() {
            // Restore clip state
            let clip_diff = current_clip - state.clip_count;

            for _ in 0..clip_diff {
                self.render_context.render.pop_stencil();
            }

            self.current_state = state;
        } else {
            println!("Trying to restore a stack without any saved state.");
        }
    }

    pub fn quadratic_curve_to(&mut self, ctrl: Position, to: Position) {
        let ctrl = ctrl + self.position;
        let to = to + self.position;

        self.path_builder.quadratic_bezier_to(ctrl, to);
    }

    pub fn bezier_curve_to(&mut self, ctrl1: Position, ctrl2: Position, to: Position) {
        let ctrl1 = ctrl1 + self.position;
        let ctrl2 = ctrl2 + self.position;
        let to = to + self.position;

        self.path_builder.cubic_bezier_to(ctrl1, ctrl2, to);
    }

    pub fn arc(&mut self, center: Position, r: f64, start_angle: Angle, end_angle: Angle) {
        let center = center + self.position;

        self.path_builder.arc(
            center,
            Dimension::new(r, r),
            start_angle,
            end_angle,
        )
    }

    pub fn arc_to(&mut self, _x1: f64, _y1: f64, _x2: f64, _y2: f64, _r: f64) {
        todo!()
    }

    pub fn set_text_align(&mut self, alignment: Alignment) {
        self.current_state.text_alignment = alignment;
    }

    pub fn fill_text(&mut self, text: &str, x: Scalar, y: Scalar) {
        let text_id = TextId::new();

        let text_style = TextStyle {
            family: "Noto Sans".to_string(),
            font_size: 14,
            line_height: 1.0,
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
            text_decoration: TextDecoration::None,
            color: None,
            wrap: Wrap::Character,
        };

        self.render_context.text.update(text_id, text, &text_style);
        let size = self.render_context.text.calculate_size(text_id, Dimension::new(Scalar::MAX, Scalar::MAX), self.render_context.env);

        let position = match self.current_state.text_alignment {
            Alignment::TopLeading => Position::new(x, y),
            Alignment::Top => Position::new(x - size.width / 2.0, y),
            Alignment::TopTrailing => Position::new(x - size.width, y),

            Alignment::Leading => Position::new(x, y - size.height / 2.0),
            Alignment::Center => Position::new(x - size.width / 2.0, y - size.height / 2.0),
            Alignment::Trailing => Position::new(x - size.width, y - size.height / 2.0),

            Alignment::BottomLeading => Position::new(x, y - size.height),
            Alignment::Bottom => Position::new(x - size.width / 2.0, y - size.height),
            Alignment::BottomTrailing => Position::new(x - size.width, y - size.height),

            Alignment::Custom(px, py) => Position::new(x - size.width * px, y - size.height * py),
        };

        self.render_context.text.calculate_position(text_id, position + self.position, self.render_context.env);

        let style = self.current_state.fill_color.convert(position + self.position, size);
        self.render_context.style(style, |render_context| {
            render_context.text(text_id);
        });

        self.render_context.text.remove(text_id);
    }
}

impl<'a, 'b, 'c: 'b> Drop for CanvasContext<'a, 'b, 'c> {
    fn drop(&mut self) {
        if self.current_state.clip_count > 0 {
            for _ in 0..self.current_state.clip_count {
                self.render_context.render.pop_stencil();
            }
        }
    }
}

const RADIANS_FOR_MISSING_ANGLE: f32 = 100.0;

/*impl<'a, 'b, 'c: 'b> CanvasContext<'a, 'b, 'c> {
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
    ) -> Vec<Triangle<StrokeVertex>> {

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

        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &stroke_options,
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

        triangles
    }
}*/

impl<'a, 'b, 'c: 'b> Debug for CanvasContext<'a, 'b, 'c> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanvasContext")
            .field("current_state", &self.current_state)
            .field("state_stack", &self.state_stack)
            .field("position", &self.position)
            .field("dimension", &self.dimension)
            .finish_non_exhaustive()
    }
}