use carbide_core::Color;
use crate::color::{BLUE, RED};
use crate::draw::Position;

#[derive(Debug, Clone)]
pub struct Gradient {
    pub colors: Vec<Color>,
    pub ratios: Vec<f32>,
    pub start: Position,
    pub end: Position,
}

impl Gradient {
    pub fn test() -> Self {
        Self {
            colors: vec![Color::Rgba(1.0, 0.0, 0.0, 1.0), Color::Rgba(0.0, 0.0, 1.0, 1.0)],
            ratios: vec![0.0, 1.0],
            start: Position::new(100.0, 100.0),
            end: Position::new(200.0, 200.0)
        }
    }
}