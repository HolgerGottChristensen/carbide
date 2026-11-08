/// How textures should be sampled.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum SampleType {
    Nearest,
    #[default]
    Linear,
}
