use carbide::impl_state_value;

/// How textures should be sampled.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum SampleType {
    Nearest,
    #[default]
    Linear,
}

impl_state_value!(SampleType);
