use crate::animation::AnimationManager;
use crate::draw::path::PathBuilder;
use crate::draw::stroke::{LineCap, LineJoin, StrokeDashCap, StrokeDashMode, StrokeDashPattern, StrokeOptions};
use crate::draw::{Alignment, Angle, Dimension, Position, Scalar};
use crate::math::{Deg, Matrix4, SquareMatrix, Vector3};
use crate::widget::rotation_3d_effect::{rotation_3d_matrix, DEFAULT_FOV};
use crate::draw::{Color, DrawShape};
use crate::environment::Environment;
use crate::mouse_position::MousePositionEnvironmentExt;
use crate::render::{RenderContext, Style};
use crate::state::{IntoReadState, ReadState, StateSync};
use crate::text::text_wrap::Wrap;
use crate::text::{FontSize, FontStyle, FontWeight, TextDecoration, TextStyle};
use crate::widget::ShapeStyle;
use std::fmt::{Debug, Formatter};

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
    cap_style: LineCap,
    join_style: LineJoin,
    line_width: Scalar,
    dash_pattern: Option<Vec<f64>>,
    dash_offset: Scalar,
    dash_start_cap: StrokeDashCap,
    dash_end_cap: StrokeDashCap,
    dash_mode: StrokeDashMode,
    miter_limit: Scalar,
    text_alignment: Alignment,
    /// The style `fill_text` draws with and `measure_text` measures with. Its `color` is not used:
    /// text is drawn in the fill style.
    text_style: TextStyle,
    clip_count: u32,
    /// The transform `fill`, `stroke` and `fill_text` apply to what they draw, in the canvas's
    /// own coordinates. Saved and restored with the rest of the state.
    transform: Matrix4<Scalar>,
}

impl<'a, 'b, 'c: 'b> CanvasContext<'a, 'b, 'c> {
    pub fn new(position: Position, dimension: Dimension, render_context: &'a mut RenderContext<'b, 'c>) -> CanvasContext<'a, 'b, 'c> {
        CanvasContext {
            render_context,
            current_state: ContextState {
                stroke_color: Style::Color(Color::Rgba(0.0, 0.0, 0.0, 1.0)),
                fill_color: Style::Color(Color::Rgba(0.0, 0.0, 0.0, 1.0)),
                cap_style: LineCap::Round,
                join_style: LineJoin::Round,
                line_width: 2.0,
                dash_pattern: None,
                dash_offset: 0.0,
                dash_start_cap: StrokeDashCap::None,
                dash_end_cap: StrokeDashCap::None,
                dash_mode: StrokeDashMode::Pretty,
                miter_limit: 4.0, //StrokeOptions::DEFAULT_MITER_LIMIT,
                text_alignment: Alignment::TopLeading,
                text_style: TextStyle {
                    family: "Noto Sans".to_string(),
                    font_size: 14,
                    line_height: 1.0,
                    font_style: FontStyle::Normal,
                    font_weight: FontWeight::Normal,
                    text_decoration: TextDecoration::None,
                    color: None,
                    wrap: Wrap::Character,
                },
                clip_count: 0,
                transform: Matrix4::identity(),
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

    pub fn set_line_join(&mut self, join: LineJoin) {
        self.current_state.join_style = join;
    }

    pub fn set_line_cap(&mut self, cap: LineCap) {
        self.current_state.cap_style = cap;
    }

    pub fn set_miter_limit(&mut self, limit: Scalar) {
        self.current_state.miter_limit = limit;
    }

    /// The current transform, which `fill`, `stroke` and `fill_text` apply to what they draw.
    pub fn get_transform(&self) -> Matrix4<Scalar> {
        self.current_state.transform
    }

    /// Replaces the current transform with `transform`. `fill`, `stroke` and `fill_text` apply it
    /// to what they draw, in the canvas's own coordinates, so the canvas's top left corner is the
    /// origin. It is part of the state `save` and `restore` keep.
    pub fn set_transform(&mut self, transform: Matrix4<Scalar>) {
        self.current_state.transform = transform;
    }

    /// Resets the current transform to the identity: paths are drawn as they are built.
    pub fn reset_transform(&mut self) {
        self.set_transform(Matrix4::identity());
    }

    /// Adds `transform` to the current one, by multiplying it on the right: `transform` is applied
    /// to what is drawn first, then the transforms set before it, as with an HTML canvas.
    pub fn transform(&mut self, transform: Matrix4<Scalar>) {
        self.current_state.transform = self.current_state.transform * transform;
    }

    /// Adds a translation by `offset` to the current transform (see `transform`).
    pub fn translate(&mut self, offset: Vector3<Scalar>) {
        self.transform(Matrix4::from_translation(offset));
    }

    /// Adds a scale by `factors` along x, y and z to the current transform (see `transform`).
    /// It scales around the origin, so a scale by 2 also moves what is drawn away from it.
    pub fn scale(&mut self, factors: Vector3<Scalar>) {
        self.transform(Matrix4::from_nonuniform_scale(factors.x, factors.y, factors.z));
    }

    /// Adds a rotation by `angle` around the z axis, out of the canvas, to the current transform
    /// (see `transform`). It rotates around the origin; positive angles turn from the x axis
    /// towards the y axis, clockwise on screen, as y grows downwards.
    pub fn rotate(&mut self, angle: Angle) {
        self.transform(Matrix4::from_angle_z(Deg(angle.degrees())));
    }

    /// Adds a rotation by `rotation_x` around the x axis and `rotation_y` around the y axis, seen
    /// in perspective, to the current transform (see `transform`): the same rotation the
    /// `Rotation3DEffect` widget applies, with its default field of view. It rotates around the
    /// origin, so translate to where it should turn around first, as the widget's anchor does.
    pub fn rotate_3d(&mut self, rotation_x: Angle, rotation_y: Angle) {
        self.transform(rotation_3d_matrix(Deg(rotation_x.degrees()), Deg(rotation_y.degrees()), DEFAULT_FOV));
    }

    /// Runs `draw` with the current transform applied, moved to where the canvas is drawn: paths
    /// are built with the canvas's position added (see `move_to`), so the transform is applied
    /// around that position, as the canvas's origin. Without a transform, `draw` runs as it is, so
    /// an untransformed canvas renders exactly as it did before transforms existed.
    fn with_transform(&mut self, draw: impl FnOnce(&mut RenderContext)) {
        if self.current_state.transform == Matrix4::identity() {
            draw(self.render_context);
            return;
        }

        let origin = Vector3::new(self.position.x, self.position.y, 0.0);
        let transform = Matrix4::from_translation(origin) * self.current_state.transform * Matrix4::from_translation(-origin);

        // The renderer works in `f32`; the transform is kept in `Scalar` until it is drawn.
        let transform = transform.cast::<f32>().expect("an f64 matrix always casts to f32");

        self.render_context.transform(transform, draw);
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

    /// Fills the current path, with the current transform applied (see `set_transform`).
    pub fn fill(&mut self) {
        #[derive(Clone, Debug)]
        struct FillShape {
            path: crate::draw::path::Path,
        }

        impl From<FillShape> for DrawShape {
            fn from(value: FillShape) -> Self {
                DrawShape::Path(value.path.clone())
            }
        }

        let path = self.path_builder.path().clone();
        let style = self.current_state.fill_color.convert(self.position, self.dimension);

        self.with_transform(|ctx| {
            ctx.style(style, |ctx| {
                ctx.shape(FillShape { path }, ShapeStyle::Fill)
            })
        });
    }

    /// Strokes the current path, with the current transform applied (see `set_transform`).
    pub fn stroke(&mut self) {
        #[derive(Clone, Debug)]
        struct StrokeShape {
            path: crate::draw::path::Path,
        }

        impl From<StrokeShape> for DrawShape {
            fn from(value: StrokeShape) -> Self {
                DrawShape::Path(value.path.clone())
            }
        }

        let join_style = match self.current_state.join_style {
            LineJoin::Miter => LineJoin::Miter,
            LineJoin::MiterClip { .. } => LineJoin::MiterClip { miter_limit: self.current_state.miter_limit },
            LineJoin::Round => LineJoin::Round,
            LineJoin::Bevel => LineJoin::Bevel
        };

        let stroke_options = StrokeOptions::default()
            .with_stroke_cap(self.current_state.cap_style)
            .with_stroke_width(self.current_state.line_width)
            .with_stroke_join(join_style);

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
        let style = self.current_state.stroke_color.convert(self.position, self.dimension);

        // The transform applies to the stroke as drawn, so a scale also scales the line's width,
        // as with an HTML canvas.
        self.with_transform(|ctx| {
            ctx.style(style, |ctx| {
                ctx.stroke_dash_pattern(dashes, |ctx| {
                    ctx.shape(StrokeShape { path }, stroke_options)
                })
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

    /// The style `fill_text` draws with and `measure_text` measures with: Noto Sans at 14,
    /// normal weight and style, without decoration, unless set. Saved and restored with the rest
    /// of the state.
    pub fn get_text_style(&self) -> &TextStyle {
        &self.current_state.text_style
    }

    /// Replaces the style `fill_text` draws with and `measure_text` measures with. The style's
    /// `color` is not used: text is drawn in the fill style (see `set_fill_style`).
    pub fn set_text_style(&mut self, style: TextStyle) {
        self.current_state.text_style = style;
    }

    /// Sets the font family of the text style, such as "Noto Sans".
    pub fn set_font_family(&mut self, family: impl Into<String>) {
        self.current_state.text_style.family = family.into();
    }

    /// Sets the font size of the text style.
    pub fn set_font_size(&mut self, size: FontSize) {
        self.current_state.text_style.font_size = size;
    }

    /// Sets the font weight of the text style.
    pub fn set_font_weight(&mut self, weight: FontWeight) {
        self.current_state.text_style.font_weight = weight;
    }

    /// Sets the font style of the text style: normal or italic.
    pub fn set_font_style(&mut self, style: FontStyle) {
        self.current_state.text_style.font_style = style;
    }

    /// Sets the decoration of the text style: none, underline, overline or strike-through.
    pub fn set_text_decoration(&mut self, decoration: TextDecoration) {
        self.current_state.text_style.text_decoration = decoration;
    }

    /// The size `text` takes up when drawn with `fill_text`.
    pub fn measure_text(&mut self, text: &str) -> Dimension {
        self.render_context.measure_text(text, &self.current_state.text_style, None)
    }

    /// Draws `text` at (`x`, `y`), placed by the current text alignment, with the current transform
    /// applied (see `set_transform`): a rotation turns the text around the origin, so rotate
    /// around the text's position to turn it in place.
    pub fn fill_text(&mut self, text: &str, x: Scalar, y: Scalar) {
        let text_style = self.current_state.text_style.clone();

        let size = self.render_context.measure_text(text, &text_style, None);

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

        let style = self.current_state.fill_color.convert(position + self.position, size);
        let position = position + self.position;

        self.with_transform(|render_context| {
            render_context.style(style, |render_context| {
                render_context.text(text, &text_style, position, None);
            });
        });
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