use bitflags::Flags;
use carbide::draw::ImageId;
use crate::material::material_flags::MaterialFlags;
use crate::material::normal_texture_y_direction::NormalTextureYDirection;

/// How normals should be derived
#[derive(Debug, Clone)]
pub enum NormalTexture {
    /// No normal texture.
    None,
    /// Normal stored in RGB values.
    Tricomponent(ImageId, NormalTextureYDirection),
    /// Normal stored in RG values, third value should be reconstructed.
    Bicomponent(ImageId, NormalTextureYDirection),
    /// Normal stored in Green and Alpha values, third value should be
    /// reconstructed. This is useful for storing in BC3 or BC7 compressed
    /// textures.
    BicomponentSwizzled(ImageId, NormalTextureYDirection),
}
impl Default for NormalTexture {
    fn default() -> Self {
        Self::None
    }
}

impl NormalTexture {
    pub fn to_texture(&self) -> Option<&ImageId> {
        match *self {
            Self::None => None,
            Self::Tricomponent(ref texture, _)
            | Self::Bicomponent(ref texture, _)
            | Self::BicomponentSwizzled(ref texture, _) => Some(texture),
        }
    }

    pub fn to_flags(&self) -> MaterialFlags {
        // Start with the base component flags
        let base = match self {
            Self::None => MaterialFlags::empty(),
            Self::Tricomponent(..) => MaterialFlags::empty(),
            Self::Bicomponent(..) => MaterialFlags::BICOMPONENT_NORMAL,
            Self::BicomponentSwizzled(..) => MaterialFlags::BICOMPONENT_NORMAL | MaterialFlags::SWIZZLED_NORMAL,
        };

        // Add the direction flags
        match self {
            Self::Tricomponent(_, NormalTextureYDirection::Down)
            | Self::Bicomponent(_, NormalTextureYDirection::Down)
            | Self::BicomponentSwizzled(_, NormalTextureYDirection::Down) => base | MaterialFlags::YDOWN_NORMAL,
            _ => base,
        }
    }
}