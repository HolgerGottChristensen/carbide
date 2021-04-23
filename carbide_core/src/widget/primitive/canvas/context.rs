use crate::{Point, Color};
use lyon::algorithms::path::Path;
use lyon::lyon_algorithms::path::math::point;
use crate::widget::types::shape_style::ShapeStyle;
use lyon::tessellation::{StrokeOptions, FillOptions, LineJoin, LineCap};
//use crate::draw::path_builder::PathBuilder;
use lyon::algorithms::path::builder::{PathBuilder as PB, Build, SvgPathBuilder};
use std::rc::Rc;
use crate::draw::svg_path_builder::SVGPathBuilder;

#[derive(Debug, Clone)]
pub struct Context {
    generator: Vec<ContextAction>
}


impl Context {

    pub fn new() -> Context {
        Context {
            generator: vec![ContextAction::MoveTo([0.0, 0.0])]
        }
    }

    pub fn set_line_width(&mut self, width: f64) {
        self.generator.push(ContextAction::LineWidth(width))
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

    pub fn set_fill_style(&mut self, color: Color) {
        self.generator.push(ContextAction::FillStyle(color))
    }

    pub fn set_stroke_style(&mut self, color: Color) {
        self.generator.push(ContextAction::StrokeStyle(color))
    }

    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.generator.push(ContextAction::Rect(x, y, width, height))
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
        self.generator.push(ContextAction::MoveTo([x, y]))
    }

    pub fn close_path(&mut self) {
        self.generator.push(ContextAction::Close)
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        self.generator.push(ContextAction::LineTo([x, y]))
    }

    pub fn clip(&mut self) {
        todo!()
    }

    pub fn quadratic_curve_to(&mut self, ctrl: Point, to: Point) {
        self.generator.push(ContextAction::QuadraticBezierTo {ctrl, to})
    }

    pub fn bezier_curve_to(&mut self, ctrl1: Point, ctrl2: Point, to: Point) {
        self.generator.push(ContextAction::CubicBezierTo {
            ctrl1,
            ctrl2,
            to
        })
    }

    pub fn arc(&mut self, x: f64, y: f64, r: f64, start_angle: f64, end_angle: f64) {
        self.generator.push(ContextAction::Arc {x, y, r, start_angle, end_angle})
    }

    pub fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, r: f64) {
        self.generator.push(ContextAction::ArcTo {x1, y1, x2, y2, r})
    }

    pub fn to_paths(&self, offset: Point) -> Vec<(Path, ShapeStyleWithOptions)> {
        let mut current_stroke_color = Color::Rgba(0.0,0.0,0.0,1.0);
        let mut current_fill_color = Color::Rgba(0.0,0.0,0.0,1.0);
        let mut current_cap_style = LineCap::Round;
        let mut current_join_style = LineJoin::Round;
        let mut current_line_width = 2.0;
        let mut current_miter_limit = StrokeOptions::DEFAULT_MITER_LIMIT;
        let mut paths: Vec<(Path, ShapeStyleWithOptions)> = vec![];
        let mut current_builder = SVGPathBuilder::new();
        let mut current_builder_begun = false;
        let mut current_builder_closed = false;

        let offset_point = |p: [f64; 2]| {
            point(p[0] as f32 + offset[0] as f32, p[1] as f32 + offset[1] as f32)
        };

        for action in &self.generator {
            if !current_builder_begun {
                current_builder = SVGPathBuilder::new();
                current_builder_begun = true;
                current_builder_closed = false;
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
                    current_builder.cubic_bezier_to(offset_point(*ctrl1), offset_point(*ctrl2), offset_point(*to));
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
                ContextAction::Arc { .. } => {
                    todo!()
                }
                ContextAction::ArcTo { .. } => {
                    todo!()
                }
                ContextAction::FillStyle(color) => {
                    current_fill_color = *color;
                }
                ContextAction::StrokeStyle(color) => {
                    current_stroke_color = *color;
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
                    paths.push((path, ShapeStyleWithOptions::Stroke(stroke_options, color)));
                }
            }
        }

        paths

    }
}

pub enum ShapeStyleWithOptions {
    Fill(FillOptions, Color),
    Stroke(StrokeOptions, Color),
}

#[derive(Debug, Clone)]
pub enum ContextAction {
    MoveTo(Point),
    LineTo(Point),
    QuadraticBezierTo {ctrl: Point, to: Point},
    CubicBezierTo {ctrl1: Point, ctrl2: Point, to: Point},
    Fill,
    Stroke,
    Close,
    LineWidth(f64),
    LineJoin(LineJoin),
    LineCap(LineCap),
    MiterLimit(f64),
    Rect(f64, f64, f64, f64),
    BeginPath,
    Arc {x: f64, y: f64, r: f64, start_angle: f64, end_angle: f64},
    ArcTo {x1: f64, y1: f64, x2: f64, y2: f64, r: f64},
    FillStyle(Color),
    StrokeStyle(Color),
}
