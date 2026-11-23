//! Turns SVG documents into render instructions for the [Image] widget. Only compiled with the
//! `svg` feature, since parsing pulls in `carbide_usvg`.

use super::Image;
use crate::color::{rgb_bytes, BLUE};
use crate::draw::fill::FillOptions;
use crate::draw::path::PathInstruction;
use crate::draw::stroke::{LineCap, LineJoin, StrokeOptions};
use crate::draw::{Dimension, DrawOptions, DrawShape, ImageId, Position, Scalar};
use crate::layout::LayoutContext;
use crate::render::{RenderInstruction, RenderInstructionValue, Style};
use crate::state::{ReadState, ReadStateExt};
use carbide_usvg::tiny_skia_path::PathSegment;
use carbide_usvg::{Document, Fill, Group, Options, Paint, PaintOrder, Stroke, Tree};
use cgmath::{Matrix4, Vector4};

impl<Id: ReadState<T=ImageId>, C: ReadState<T=Style>> Image<Id, C> {
    /// Parses `source` as an SVG document and registers the result as a vector image for `image_id`.
    pub(super) fn load_svg(&self, ctx: &mut LayoutContext, image_id: &ImageId, source: &str) {
        let options = Options::default();
        let doc = Document::from_str(source, &options);
        let tree = doc.to_tree(&options);

        let description = self.construct_render_instructions(&tree);

        ctx.image.update_vector(
            image_id,
            description,
            Dimension::new(tree.size().width() as Scalar, tree.size().height() as Scalar),
            ctx.env
        );
    }

    fn construct_render_instructions(&self, tree: &Tree) -> Vec<RenderInstruction> {
        let mut description = vec![];

        self.render_instructions_from_group(tree.root(), &mut description);

        description
    }

    fn render_instructions_from_group(&self, group: &Group, description: &mut Vec<RenderInstruction>) {

        let group_transform = group.transform();

        let matrix = Matrix4::from_cols(
            Vector4::new(group_transform.sx, group_transform.ky, 0.0, 0.0),
            Vector4::new(group_transform.kx, group_transform.sy, 0.0, 0.0),
            Vector4::new(0.0, 0.0, 1.0, 0.0),
            Vector4::new(group_transform.tx, group_transform.ty, 0.0, 1.0)
        );

        description.push(RenderInstruction::PushTransform { transform: matrix });

        for child in group.children() {
            self.render_instructions_from_node(child, description)
        }

        description.push(RenderInstruction::PopTransform);
    }

    fn render_instructions_from_path(&self, path: &carbide_usvg::Path, description: &mut Vec<RenderInstruction>) {
        let mut path_instructions = vec![];

        for segment in path.data().segments() {
            match segment {
                PathSegment::MoveTo(point) => path_instructions.push(PathInstruction::MoveTo {
                    to: Position::new(point.x as f64, point.y as f64)
                }),
                PathSegment::LineTo(point) => path_instructions.push(PathInstruction::LineTo {
                    to: Position::new(point.x as f64, point.y as f64)
                }),
                PathSegment::QuadTo(ctrl, point) => path_instructions.push(PathInstruction::QuadraticBezierTo {
                    ctrl: Position::new(ctrl.x as f64, ctrl.y as f64),
                    to: Position::new(point.x as f64, point.y as f64)
                }),
                PathSegment::CubicTo(ctrl1, ctrl2, point) => path_instructions.push(PathInstruction::CubicBezierTo {
                    ctrl1: Position::new(ctrl1.x as f64, ctrl1.y as f64),
                    ctrl2: Position::new(ctrl2.x as f64, ctrl2.y as f64),
                    to: Position::new(point.x as f64, point.y as f64)
                }),
                PathSegment::Close => path_instructions.push(PathInstruction::Close),
            }
        }

        match path.paint_order() {
            PaintOrder::FillAndStroke => {
                if let Some(fill) = path.fill() {
                    self.fill_path(description, &path_instructions, fill);
                }
                if let Some(stroke) = path.stroke() {
                    self.stroke_path(description, &path_instructions, stroke);
                }
            }
            PaintOrder::StrokeAndFill => {
                if let Some(stroke) = path.stroke() {
                    self.stroke_path(description, &path_instructions, stroke);
                }
                if let Some(fill) = path.fill() {
                    self.fill_path(description, &path_instructions, fill);
                }
            }
        }
    }

    fn fill_path(&self, description: &mut Vec<RenderInstruction>, path_instructions: &Vec<PathInstruction>, fill: &Fill) {
        let style = match fill.paint() {
            Paint::CurrentColor => self.color.as_ref()
                .map(|a| RenderInstructionValue::Variable(a.as_dyn_read()))
                .unwrap_or(RenderInstructionValue::Constant(Style::Color(BLUE))),
            Paint::Color(c) => RenderInstructionValue::Constant(Style::Color(rgb_bytes(c.red, c.green, c.blue))),
            Paint::LinearGradient(_) => RenderInstructionValue::Constant(Style::Color(BLUE)),
            Paint::RadialGradient(_) => RenderInstructionValue::Constant(Style::Color(BLUE)),
            Paint::Pattern(_) => RenderInstructionValue::Constant(Style::Color(BLUE))
        };

        description.push(RenderInstruction::PushStyle { style });

        description.push(RenderInstruction::Shape {
            shape: DrawShape::Path(crate::draw::path::Path { instructions: path_instructions.clone() }),
            options: DrawOptions::Fill(FillOptions::default())
        });

        description.push(RenderInstruction::PopStyle);
    }

    fn stroke_path(&self, description: &mut Vec<RenderInstruction>, path_instructions: &Vec<PathInstruction>, stroke: &Stroke) {
        let style = match stroke.paint() {
            Paint::CurrentColor => self.color.as_ref()
                .map(|a| RenderInstructionValue::Variable(a.as_dyn_read()))
                .unwrap_or(RenderInstructionValue::Constant(Style::Color(BLUE))),
            Paint::Color(c) => RenderInstructionValue::Constant(Style::Color(rgb_bytes(c.red, c.green, c.blue))),
            Paint::LinearGradient(_) => RenderInstructionValue::Constant(Style::Color(BLUE)),
            Paint::RadialGradient(_) => RenderInstructionValue::Constant(Style::Color(BLUE)),
            Paint::Pattern(_) => RenderInstructionValue::Constant(Style::Color(BLUE))
        };

        let cap = match stroke.linecap() {
            carbide_usvg::LineCap::Butt => LineCap::Butt,
            carbide_usvg::LineCap::Round => LineCap::Round,
            carbide_usvg::LineCap::Square => LineCap::Square
        };

        let join = match stroke.linejoin() {
            carbide_usvg::LineJoin::Miter => LineJoin::Miter,
            carbide_usvg::LineJoin::MiterClip => LineJoin::MiterClip { miter_limit: stroke.miterlimit().get() as Scalar },
            carbide_usvg::LineJoin::Round => LineJoin::Round,
            carbide_usvg::LineJoin::Bevel => LineJoin::Bevel,
        };


        description.push(RenderInstruction::PushStyle { style });

        description.push(RenderInstruction::Shape {
            shape: DrawShape::Path(crate::draw::path::Path { instructions: path_instructions.clone() }),
            options: DrawOptions::Stroke(StrokeOptions::default()
                .with_stroke_cap(cap)
                .with_stroke_join(join)
                .with_stroke_width(stroke.width().get() as Scalar)
            )
        });

        description.push(RenderInstruction::PopStyle);
    }

    fn render_instructions_from_node(&self, node: &carbide_usvg::Node, description: &mut Vec<RenderInstruction>) {
        match node {
            carbide_usvg::Node::Group(g) => self.render_instructions_from_group(g, description),
            carbide_usvg::Node::Path(p) => self.render_instructions_from_path(p, description),
            carbide_usvg::Node::Image(_) => {}
            carbide_usvg::Node::Text(_) => {}
        }
    }
}

pub(super) static MISSING_SVG: &str = r#"
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
        <line x1="2" x2="22" y1="2" y2="22" />
        <path d="M10.41 10.41a2 2 0 1 1-2.83-2.83" />
        <line x1="13.5" x2="6" y1="13.5" y2="21" />
        <line x1="18" x2="21" y1="12" y2="15" />
        <path d="M3.59 3.59A1.99 1.99 0 0 0 3 5v14a2 2 0 0 0 2 2h14c.55 0 1.052-.22 1.41-.59" />
        <path d="M21 15V5a2 2 0 0 0-2-2H9" />
    </svg>
"#;
