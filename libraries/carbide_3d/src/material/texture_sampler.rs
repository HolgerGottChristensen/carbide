//! How a material's map is sampled: what happens to texture coordinates outside 0 to 1, and how
//! texels are filtered, as glTF's per-texture `sampler`, three.js's `Texture.wrapS`, `magFilter`
//! and `minFilter`, and Qt Quick 3D's `Texture` tiling modes and filters.

use crate::material::sample_type::SampleType;

/// What a map does with texture coordinates outside 0 to 1, along one axis.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum Wrap {
    /// The image repeats: 1.25 reads as 0.25. glTF's `REPEAT`.
    #[default]
    Repeat,
    /// The edge texel continues: 1.25 reads as 1. glTF's `CLAMP_TO_EDGE`.
    ClampToEdge,
    /// The image repeats, every other copy mirrored: 1.25 reads as 0.75. glTF's
    /// `MIRRORED_REPEAT`.
    MirroredRepeat,
}

/// How texels are combined: the nearest one, or a blend of the four around.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest,
    Linear,
}

/// How a map is filtered: up close (`mag`), from afar (`min`), and between its mip levels
/// (`mipmap`, `None` to read only the full-size image, as glTF's `NEAREST` and `LINEAR`
/// minification filters).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Filtering {
    pub mag: Filter,
    pub min: Filter,
    pub mipmap: Option<Filter>,
}

impl Filtering {
    /// Linear everywhere, between mip levels too: smooth, and the default.
    pub const LINEAR: Filtering = Filtering { mag: Filter::Linear, min: Filter::Linear, mipmap: Some(Filter::Linear) };
    /// Nearest everywhere: sharp texels, for pixel art.
    pub const NEAREST: Filtering = Filtering { mag: Filter::Nearest, min: Filter::Nearest, mipmap: Some(Filter::Nearest) };
}

impl From<SampleType> for Filtering {
    fn from(sample_type: SampleType) -> Self {
        match sample_type {
            SampleType::Linear => Filtering::LINEAR,
            SampleType::Nearest => Filtering::NEAREST,
        }
    }
}

/// How a map is sampled: its wrap along u and v, and its filtering, once a map's own filtering
/// or the material's default is settled.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TextureSampler {
    pub wrap_u: Wrap,
    pub wrap_v: Wrap,
    pub filtering: Filtering,
}

impl Default for TextureSampler {
    fn default() -> Self {
        TextureSampler { wrap_u: Wrap::Repeat, wrap_v: Wrap::Repeat, filtering: Filtering::LINEAR }
    }
}
