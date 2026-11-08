use crate::draw::draw_gradient::DrawGradient;
use crate::draw::Color;

#[derive(Debug, Clone, PartialEq)]
pub enum DrawStyle {
    Color(Color),
    Gradient(DrawGradient),
    MultiGradient(Vec<DrawGradient>),
}