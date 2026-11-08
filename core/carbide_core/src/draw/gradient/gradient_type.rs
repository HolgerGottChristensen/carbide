/// The different types of gradients in carbide.
#[derive(Debug, Clone, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
    Diamond,
    Conic,
}