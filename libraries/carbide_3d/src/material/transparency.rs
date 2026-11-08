/// How transparency should be handled in a material.
#[derive(Debug, Copy, Clone, PartialEq)]
#[derive(Default)]
pub enum Transparency {
    /// Alpha is completely ignored.
    #[default]
    Opaque,
    /// Pixels with alpha less than `cutout` is discorded.
    Cutout { cutout: f32 },
    /// Alpha is blended.
    Blend,
}
