use carbide::environment::Environment;
use carbide_core::draw::{Position, Scalar};
use carbide_core::widget::canvas::{CanvasContext, LineCap, LineJoin};
use crate::element::cubic_interpolation_mode::CubicInterpolationMode;
use crate::element::Element;
use crate::element::span_gaps::SpanGaps;
use crate::element::stepped::Stepped;

pub struct Line {
    position: Position,
    points: Vec<Position>,
    border_cap_style: LineCap,
    border_join_style: LineJoin,
    border_width: Scalar,
    cap_bezier_points: bool,
    cubic_interpolation_mode: CubicInterpolationMode,
    fill: bool,
    span_gaps: SpanGaps,
    stepped: Stepped
}

impl Element for Line {
    fn x(&self) -> Scalar {
        self.position.x
    }

    fn y(&self) -> Scalar {
        self.position.y
    }

    fn draw(&self, ctx: &mut CanvasContext, env: &mut Environment) {
        todo!()
    }
}