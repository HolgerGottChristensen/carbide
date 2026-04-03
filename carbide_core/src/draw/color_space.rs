use carbide_derive::Id;

#[derive(Debug, Copy, Clone, PartialEq, Id, Eq, Hash)]
pub enum ColorSpace {
    Linear,
    OkLAB,
    Srgb,
    Xyz,
    Cielab,
    HSL,
}