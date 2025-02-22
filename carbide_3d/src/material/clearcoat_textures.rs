use crate::material::material_flags::MaterialFlags;
use carbide::draw::ImageId;

/// How clearcoat values should be derived.
#[derive(Debug, Clone)]
pub enum ClearcoatTextures {
    GltfCombined {
        /// Texture with Clearcoat in R, and Clearcoat Roughness in G
        texture: Option<ImageId>,
    },
    GltfSplit {
        /// Texture with Clearcoat in R
        clearcoat_texture: Option<ImageId>,
        /// Texture with Clearcoat Roughness in G
        clearcoat_roughness_texture: Option<ImageId>,
    },
    BWSplit {
        /// Texture with Clearcoat in R
        clearcoat_texture: Option<ImageId>,
        /// Texture with Clearcoat Roughness in R
        clearcoat_roughness_texture: Option<ImageId>,
    },
    None,
}

impl ClearcoatTextures {
    pub fn to_clearcoat_texture(&self) -> Option<&ImageId> {
        match *self {
            Self::GltfCombined { texture: Some(ref texture) } => Some(texture),
            Self::GltfSplit { clearcoat_texture: Some(ref texture), .. } => Some(texture),
            Self::BWSplit { clearcoat_texture: Some(ref texture), .. } => Some(texture),
            _ => None,
        }
    }

    pub fn to_clearcoat_roughness_texture(&self) -> Option<&ImageId> {
        match *self {
            Self::GltfCombined { .. } => None,
            Self::GltfSplit { clearcoat_roughness_texture: Some(ref texture), .. } => Some(texture),
            Self::BWSplit { clearcoat_roughness_texture: Some(ref texture), .. } => Some(texture),
            _ => None,
        }
    }

    pub fn to_flags(&self) -> MaterialFlags {
        match self {
            Self::GltfCombined { .. } => MaterialFlags::CC_GLTF_COMBINED,
            Self::GltfSplit { .. } => MaterialFlags::CC_GLTF_SPLIT,
            Self::BWSplit { .. } => MaterialFlags::CC_BW_SPLIT,
            // Use CC_GLTF_COMBINED so shader only checks clear coat texture, then bails
            Self::None => MaterialFlags::CC_GLTF_COMBINED,
        }
    }
}
impl Default for ClearcoatTextures {
    fn default() -> Self {
        Self::None
    }
}