/// How textures should be sampled.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SampleType {
    Nearest,
    Linear,
}
impl Default for SampleType {
    fn default() -> Self {
        Self::Linear
    }
}