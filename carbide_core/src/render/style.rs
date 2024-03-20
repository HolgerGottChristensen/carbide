use crate::draw::{Color, Dimension, DrawGradient, DrawStyle, Position};
use crate::widget::Gradient;

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Color(Color),
    Gradient(Gradient),
    MultiGradient(Vec<Gradient>),
}

impl Style {
    pub fn convert(&self, position: Position, dimension: Dimension) -> DrawStyle {
        match self {
            Style::Color(c) => DrawStyle::Color(c.clone()),
            Style::Gradient(g) => DrawStyle::Gradient(DrawGradient::convert(g.clone(), position, dimension)),
            Style::MultiGradient(_) => todo!(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::Color(Color::default())
    }
}

impl From<Color> for Style {
    fn from(c: Color) -> Self {
        Style::Color(c)
    }
}