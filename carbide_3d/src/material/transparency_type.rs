use crate::material::transparency::Transparency;
use crate::sorting::Sorting;

/// The type of transparency in a material.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransparencyType {
    /// Alpha is completely ignored.
    Opaque,
    /// Alpha less than a specified value is discorded.
    Cutout,
    /// Alpha is blended.
    Blend,
}
impl From<Transparency> for TransparencyType {
    fn from(t: Transparency) -> Self {
        match t {
            Transparency::Opaque => Self::Opaque,
            Transparency::Cutout { .. } => Self::Cutout,
            Transparency::Blend => Self::Blend,
        }
    }
}
impl TransparencyType {
    pub fn to_debug_str(self) -> &'static str {
        match self {
            TransparencyType::Opaque => "opaque",
            TransparencyType::Cutout => "cutout",
            TransparencyType::Blend => "blend",
        }
    }

    pub fn to_sorting(self) -> Sorting {
        match self {
            Self::Opaque | Self::Cutout => Sorting::OPAQUE,
            Self::Blend => Sorting::BLENDING,
        }
    }
}

#[allow(clippy::cmp_owned)] // This thinks making a temporary TransparencyType is the end of the world
impl PartialEq<Transparency> for TransparencyType {
    fn eq(&self, other: &Transparency) -> bool {
        *self == Self::from(*other)
    }
}

#[allow(clippy::cmp_owned)]
impl PartialEq<TransparencyType> for Transparency {
    fn eq(&self, other: &TransparencyType) -> bool {
        TransparencyType::from(*self) == *other
    }
}