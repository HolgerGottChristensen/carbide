/// How transparency should be handled in a material.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Transparency {
    /// Alpha is completely ignored.
    Opaque,
    /// Pixels with alpha less than `cutout` is discorded.
    Cutout { cutout: f32 },
    /// Alpha is blended.
    Blend,
}
impl Default for Transparency {
    fn default() -> Self {
        Self::Opaque
    }
}