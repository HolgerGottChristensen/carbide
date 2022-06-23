use crate::Line;

#[derive(Clone, Debug)]
pub enum Guide {
    Vertical(f64),
    Horizontal(f64),
    Directional(Line)
}