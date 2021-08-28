#[derive(Debug, Clone)]
pub enum BlurType {
    /// A mean blur with the specified radius
    Mean(u32),
    /// A gaussian blur with the specified sigma
    Gaussian(f32),
}