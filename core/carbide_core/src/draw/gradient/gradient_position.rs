use crate::draw::{Alignment, Position};

#[derive(Debug, Clone, PartialEq)]
pub enum GradientPosition {
    Absolute(Position),
    Relative(f64, f64),
    Alignment(Alignment),
}