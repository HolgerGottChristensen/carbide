use crate::Color;
use crate::widget::Gradient;

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Color(Color),
    Gradient(Gradient),
    MultiGradient(Vec<Gradient>),
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