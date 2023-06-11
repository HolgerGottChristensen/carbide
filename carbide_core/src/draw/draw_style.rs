use carbide_core::draw::draw_gradient::DrawGradient;
use crate::Color;

#[derive(Debug, Clone, PartialEq)]
pub enum DrawStyle {
    Color(Color),
    Gradient(DrawGradient),
    MultiGradient(Vec<DrawGradient>),
}