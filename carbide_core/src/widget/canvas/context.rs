//use crate::draw::path_builder::PathBuilder;
use lyon::algorithms::path::builder::{Build, SvgPathBuilder};
use lyon::algorithms::path::Path;
use lyon::lyon_algorithms::path::math::point;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, LineCap, LineJoin, StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers};

use carbide_core::state::AnyReadState;

use crate::draw::{Alignment, Dimension, Position, Scalar, StrokeDashCap, StrokeDashPattern};
use crate::draw::Color;
use crate::draw::shape::triangle::Triangle;
use crate::draw::svg_path_builder::SVGPathBuilder;
use crate::environment::Environment;
use crate::render::{RenderContext, Style};
use crate::state::{IntoReadState, ReadState};
use crate::state::ReadStateExtNew;

#[derive(Debug, Clone)]
pub struct CanvasContext {
    generator: Vec<ContextAction>,
    position: Position,
    dimension: Dimension,
}

impl CanvasContext {
    pub fn new(position: Position, dimension: Dimension) -> CanvasContext {
        CanvasContext {
            generator: vec![ContextAction::MoveTo(Position::new(0.0, 0.0))],
            position,
            dimension,
        }
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    pub fn append(&mut self, mut other: CanvasContext) {
        self.generator.append(&mut other.generator);
    }

    pub fn set_line_width(&mut self, width: f64) {
        self.generator.push(ContextAction::LineWidth(width))
    }

    pub fn set_dash_pattern(&mut self, pattern: Option<Vec<f64>>) {
        self.generator.push(ContextAction::LineDashPattern(pattern));
    }

    pub fn set_dash_offset(&mut self, offset: f64) {
        self.generator.push(ContextAction::LineDashOffset(offset));
    }

    pub fn set_line_join(&mut self, join: LineJoin) {
        self.generator.push(ContextAction::LineJoin(join))
    }

    pub fn set_line_cap(&mut self, cap: LineCap) {
        self.generator.push(ContextAction::LineCap(cap))
    }

    pub fn set_miter_limit(&mut self, limit: f64) {
        self.generator.push(ContextAction::MiterLimit(limit))
    }

    pub fn set_fill_style<C: IntoReadState<Style>>(&mut self, style: C) {
        self.generator.push(ContextAction::FillStyle(style.into_read_state().as_dyn_read()))
    }

    pub fn set_stroke_style<C: IntoReadState<Style>>(&mut self, style: C) {
        self.generator
            .push(ContextAction::StrokeStyle(style.into_read_state().as_dyn_read()))
    }

    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.generator
            .push(ContextAction::Rect(x, y, width, height))
    }

    /// x, y is the top left corner of the box enclosing the circle
    pub fn circle(&mut self, x: f64, y: f64, diameter: f64) {
        self.move_to(x, y + diameter / 2.0);
        self.arc(
            x,
            y,
            diameter / 2.0,
            0.0,
            360.0,
        );
        self.move_to(0.0, 0.0);
    }

    pub fn clear_rect(&mut self) {
        todo!()
    }

    pub fn fill(&mut self) {
        self.generator.push(ContextAction::Fill)
    }

    pub fn stroke(&mut self) {
        self.generator.push(ContextAction::Stroke)
    }

    pub fn begin_path(&mut self) {
        if let Some(ContextAction::MoveTo(_)) = self.generator.last() {
            self.generator.pop();
        }
        self.generator.push(ContextAction::BeginPath)
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        if let Some(ContextAction::MoveTo(_)) = self.generator.last() {
            self.generator.pop();
        }
        self.generator
            .push(ContextAction::MoveTo(Position::new(x, y)))
    }

    pub fn close_path(&mut self) {
        self.generator.push(ContextAction::Close)
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        self.generator
            .push(ContextAction::LineTo(Position::new(x, y)))
    }

    pub fn add_lines(&mut self, lines: impl IntoIterator<Item=Position>) {
        self.generator
            .extend(lines.into_iter().map(|pos| ContextAction::LineTo(pos)))
    }

    pub fn clip(&mut self) {
        self.generator.push(ContextAction::Clip)
    }

    pub fn save(&mut self) {
        self.generator.push(ContextAction::Save)
    }

    pub fn restore(&mut self) {
        self.generator.push(ContextAction::Restore)
    }

    pub fn quadratic_curve_to(&mut self, ctrl: Position, to: Position) {
        self.generator
            .push(ContextAction::QuadraticBezierTo { ctrl, to })
    }

    pub fn bezier_curve_to(&mut self, ctrl1: Position, ctrl2: Position, to: Position) {
        self.generator
            .push(ContextAction::CubicBezierTo { ctrl1, ctrl2, to })
    }

    pub fn arc(&mut self, x: f64, y: f64, r: f64, start_angle: f64, end_angle: f64) {
        self.generator.push(ContextAction::Arc {
            x,
            y,
            r,
            start_angle,
            end_angle,
        })
    }

    pub fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, r: f64) {
        self.generator
            .push(ContextAction::ArcTo { x1, y1, x2, y2, r })
    }

    pub fn set_text_align(&mut self, alignment: Alignment) {
        self.generator.push(ContextAction::TextAlignment(alignment));
    }

    pub fn fill_text(&mut self, text: String, x: Scalar, y: Scalar) {
        self.generator.push(ContextAction::FillText(text, Position::new(x, y)))
    }

    pub fn to_paths(&self, offset: Position, env: &mut Environment) -> Vec<(Path, ShapeStyleWithOptions)> {
        let mut current_stroke_color = Style::Color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
        let mut current_fill_color = Style::Color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
        let mut current_cap_style = LineCap::Round;
        let mut current_join_style = LineJoin::Round;
        let mut current_line_width = 2.0;
        let mut current_dash_pattern = None;
        let mut current_dash_offset = 0.0;
        let mut current_miter_limit = StrokeOptions::DEFAULT_MITER_LIMIT;
        let mut current_text_alignment = Alignment::Leading;
        let mut paths: Vec<(Path, ShapeStyleWithOptions)> = vec![];
        let mut current_builder = SVGPathBuilder::new();
        let mut current_builder_begun = false;

        let mut clip_stack = vec![];

        let mut save_stack = vec![];

        struct SaveState {
            clip_stack_index: usize,
        }

        let offset_point =
            |p: Position| point(p.x as f32 + offset.x as f32, p.y as f32 + offset.y as f32);

        for action in &self.generator {
            if !current_builder_begun {
                current_builder = SVGPathBuilder::new();
                current_builder_begun = true;
            }

            match action {
                ContextAction::MoveTo(point) => {
                    current_builder.move_to(offset_point(*point));
                }
                ContextAction::LineTo(point) => {
                    current_builder.line_to(offset_point(*point));
                }
                ContextAction::QuadraticBezierTo { ctrl, to } => {
                    current_builder.quadratic_bezier_to(offset_point(*ctrl), offset_point(*to));
                }
                ContextAction::CubicBezierTo { ctrl1, ctrl2, to } => {
                    current_builder.cubic_bezier_to(
                        offset_point(*ctrl1),
                        offset_point(*ctrl2),
                        offset_point(*to),
                    );
                }
                ContextAction::Close => {
                    current_builder.close();
                }
                ContextAction::LineWidth(width) => {
                    current_line_width = *width;
                }
                ContextAction::LineJoin(join) => {
                    current_join_style = *join;
                }
                ContextAction::LineCap(cap) => {
                    current_cap_style = *cap;
                }
                ContextAction::MiterLimit(limit) => {
                    current_miter_limit = *limit as f32;
                }
                ContextAction::Rect(_, _, _, _) => {
                    todo!()
                }
                ContextAction::BeginPath => {
                    current_builder_begun = false;
                }
                ContextAction::Arc {
                    x,
                    y,
                    r,
                    start_angle,
                    end_angle,
                } => {
                    let sweep_angle = *end_angle - *start_angle;

                    current_builder.arc(
                        offset_point(Position::new(*x, *y)),
                        Dimension::new(*r, *r),
                        sweep_angle as f32,
                        *start_angle as f32,
                    )
                }
                ContextAction::ArcTo { x1: _x1, y1: _y1, x2: _x2, y2: _y2, r: _r } => {
                    todo!()
                }
                ContextAction::FillStyle(color) => {
                    color.clone().sync(env);
                    current_fill_color = color.value().clone();
                }
                ContextAction::StrokeStyle(color) => {
                    color.clone().sync(env);
                    current_stroke_color = color.value().clone();
                }
                ContextAction::Fill => {
                    let fill_options = FillOptions::default();
                    let color = current_fill_color.clone();
                    let path = current_builder.clone().build();
                    paths.push((path, ShapeStyleWithOptions::Fill(fill_options, color)));
                }
                ContextAction::Stroke => {
                    let stroke_options = StrokeOptions::default()
                        .with_line_cap(current_cap_style)
                        .with_line_width(current_line_width as f32)
                        .with_miter_limit(current_miter_limit)
                        .with_line_join(current_join_style);
                    let color = current_stroke_color.clone();
                    let path = current_builder.clone().build();
                    let dashes = current_dash_pattern.as_ref().map(|pattern: &Vec<f64>| {
                        StrokeDashPattern {
                            pattern: pattern.clone(),
                            offset: current_dash_offset,
                            start_cap: StrokeDashCap::None,
                            end_cap: StrokeDashCap::None,
                        }
                    });
                    paths.push((path, ShapeStyleWithOptions::Stroke(stroke_options, color, dashes)));
                }
                ContextAction::LineDashOffset(offset) => {
                    current_dash_offset = *offset;
                }
                ContextAction::LineDashPattern(pattern) => {
                    current_dash_pattern = pattern.clone();
                }
                ContextAction::Clip => {
                    let path = current_builder.clone().build();
                    clip_stack.push(path.clone());
                    paths.push((path, ShapeStyleWithOptions::Clip));
                }
                ContextAction::Save => {
                    save_stack.push(SaveState {
                        clip_stack_index: clip_stack.len(),
                    })
                }
                ContextAction::Restore => {
                    if let Some(state) = save_stack.pop() {
                        // Restore clip state
                        let clip_diff = save_stack.len() - state.clip_stack_index;

                        for _ in 0..clip_diff {
                            if let Some(clip) = clip_stack.pop() {
                                paths.push((clip, ShapeStyleWithOptions::UnClip));
                            }
                        }

                    } else {
                        println!("Trying to restore a stack without any saved state.");
                    }
                }
                ContextAction::TextAlignment(alignment) => {
                    current_text_alignment = *alignment;
                }
            }
        }

        paths
    }
}

impl CanvasContext {
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

    pub fn render(&self, render_context: &mut RenderContext) {
        let paths = self.to_paths(self.position, render_context.env);

        let mut clip_counter = 0;

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
                ShapeStyleWithOptions::Clip => {
                    render_context.render.stencil(&self.get_fill_geometry(path, FillOptions::default()));
                    clip_counter += 1;
                }
                ShapeStyleWithOptions::UnClip => {
                    render_context.render.pop_stencil();
                    clip_counter -= 1;
                }
            }
        }

        if clip_counter > 0 {
            for _ in 0..clip_counter {
                render_context.render.pop_stencil();
            }
        }
    }
}

#[derive(Debug)]
pub enum ShapeStyleWithOptions {
    Fill(FillOptions, Style),
    Stroke(StrokeOptions, Style, Option<StrokeDashPattern>),
    Clip,
    UnClip
}

#[derive(Debug, Clone)]
enum ContextAction {
    MoveTo(Position),
    LineTo(Position),
    QuadraticBezierTo {
        ctrl: Position,
        to: Position,
    },
    CubicBezierTo {
        ctrl1: Position,
        ctrl2: Position,
        to: Position,
    },
    Fill,
    Stroke,
    Clip,
    Save,
    Restore,
    Close,
    LineDashOffset(f64),
    LineDashPattern(Option<Vec<f64>>),
    LineWidth(f64),
    LineJoin(LineJoin),
    LineCap(LineCap),
    MiterLimit(f64),
    Rect(f64, f64, f64, f64),
    BeginPath,
    Arc {
        x: f64,
        y: f64,
        r: f64,
        start_angle: f64,
        end_angle: f64,
    },
    ArcTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        r: f64,
    },
    TextAlignment(Alignment),
    FillText(String, Position),
    FillStyle(Box<dyn AnyReadState<T=Style>>),
    StrokeStyle(Box<dyn AnyReadState<T=Style>>),
}
