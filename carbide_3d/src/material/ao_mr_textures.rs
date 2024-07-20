use carbide::draw::ImageId;
use crate::material::material_flags::MaterialFlags;

/// How the Ambient Occlusion, Metalic, and Roughness values should be
/// determined.
#[derive(Debug, Clone)]
pub enum AoMRTextures {
    None,
    Combined {
        /// Texture with Ambient Occlusion in R, Roughness in G, and Metallic in
        /// B
        texture: Option<ImageId>,
    },
    SwizzledSplit {
        /// Texture with Ambient Occlusion in R
        ao_texture: Option<ImageId>,
        /// Texture with Roughness in G and Metallic in B
        mr_texture: Option<ImageId>,
    },
    Split {
        /// Texture with Ambient Occlusion in R
        ao_texture: Option<ImageId>,
        /// Texture with Roughness in R and Metallic in G
        mr_texture: Option<ImageId>,
    },
    BWSplit {
        /// Texture with Ambient Occlusion in R
        ao_texture: Option<ImageId>,
        /// Texture with Metallic in R
        m_texture: Option<ImageId>,
        /// Texture with Roughness in R
        r_texture: Option<ImageId>,
    },
}

impl AoMRTextures {
    pub fn to_roughness_texture(&self) -> Option<&ImageId> {
        match *self {
            Self::Combined { texture: Some(ref texture) } => Some(texture),
            Self::SwizzledSplit { mr_texture: Some(ref texture), .. } => Some(texture),
            Self::Split { mr_texture: Some(ref texture), .. } => Some(texture),
            Self::BWSplit { r_texture: Some(ref texture), .. } => Some(texture),
            _ => None,
        }
    }

    pub fn to_metallic_texture(&self) -> Option<&ImageId> {
        match *self {
            Self::Combined { .. } => None,
            Self::SwizzledSplit { .. } => None,
            Self::Split { .. } => None,
            Self::BWSplit { m_texture: Some(ref texture), .. } => Some(texture),
            _ => None,
        }
    }

    pub fn to_ao_texture(&self) -> Option<&ImageId> {
        match *self {
            Self::Combined { .. } => None,
            Self::SwizzledSplit { ao_texture: Some(ref texture), .. } => Some(texture),
            Self::Split { ao_texture: Some(ref texture), .. } => Some(texture),
            Self::BWSplit { ao_texture: Some(ref texture), .. } => Some(texture),
            _ => None,
        }
    }

    pub fn to_flags(&self) -> MaterialFlags {
        match self {
            Self::Combined { .. } => MaterialFlags::AOMR_COMBINED,
            Self::SwizzledSplit { .. } => MaterialFlags::AOMR_SWIZZLED_SPLIT,
            Self::Split { .. } => MaterialFlags::AOMR_SPLIT,
            Self::BWSplit { .. } => MaterialFlags::AOMR_BW_SPLIT,
            // Use AOMR_COMBINED so shader only checks roughness texture, then bails
            Self::None => MaterialFlags::AOMR_COMBINED,
        }
    }
}
impl Default for AoMRTextures {
    fn default() -> Self {
        Self::None
    }
}