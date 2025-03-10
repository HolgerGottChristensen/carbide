use crate::draw::{Dimension, Position, Color};
use crate::draw::gradient::{Gradient, GradientPosition, GradientRepeat, GradientType};

#[derive(Debug, Clone, PartialEq)]
pub enum ColorSpace {
    Linear,
    OkLAB,
    Srgb,
    Xyz,
    Cielab,
    HSL,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawGradient {
    pub colors: Vec<Color>,
    pub ratios: Vec<f32>,

    pub gradient_type: GradientType,
    pub gradient_repeat: GradientRepeat,

    pub start: Position,
    pub end: Position,

    pub color_space: ColorSpace,
}

impl DrawGradient {
    pub fn convert(g: Gradient, position: Position, dimension: Dimension) -> Self {
        let start = match g.start {
            GradientPosition::Absolute(p) => p,
            GradientPosition::Alignment(l) => l.position(position, dimension, Dimension::default()),
            GradientPosition::Relative(x, y) => Position::new(
                position.x + dimension.width * x,
                position.y + dimension.height * y,
            ),
        };

        let end = match g.end {
            GradientPosition::Absolute(p) => p,
            GradientPosition::Alignment(l) => l.position(position, dimension, Dimension::default()),
            GradientPosition::Relative(x, y) => Position::new(
                position.x + dimension.width * x,
                position.y + dimension.height * y,
            ),
        };

        Self {
            colors: g.colors,
            ratios: g.ratios,
            gradient_type: g.gradient_type,
            gradient_repeat: g.gradient_repeat,
            start,
            end,
            color_space: g.color_space,
        }
    }
}
