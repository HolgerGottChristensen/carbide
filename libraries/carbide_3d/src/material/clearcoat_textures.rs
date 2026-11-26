use crate::material::material_flags::MaterialFlags;
use crate::material::material_texture::MaterialTexture;
use carbide::impl_state_value;

/// How clearcoat values should be derived.
#[derive(Debug, Clone)]
#[derive(Default)]
pub enum ClearcoatTextures {
    GltfCombined {
        /// Texture with Clearcoat in R, and Clearcoat Roughness in G
        texture: Option<MaterialTexture>,
    },
    GltfSplit {
        /// Texture with Clearcoat in R
        clearcoat_texture: Option<MaterialTexture>,
        /// Texture with Clearcoat Roughness in G
        clearcoat_roughness_texture: Option<MaterialTexture>,
    },
    BWSplit {
        /// Texture with Clearcoat in R
        clearcoat_texture: Option<MaterialTexture>,
        /// Texture with Clearcoat Roughness in R
        clearcoat_roughness_texture: Option<MaterialTexture>,
    },
    #[default]
    None,
}

impl ClearcoatTextures {
    pub fn to_clearcoat_texture(&self) -> Option<&MaterialTexture> {
        match *self {
            Self::GltfCombined { texture: Some(ref texture) } => Some(texture),
            Self::GltfSplit { clearcoat_texture: Some(ref texture), .. } => Some(texture),
            Self::BWSplit { clearcoat_texture: Some(ref texture), .. } => Some(texture),
            _ => None,
        }
    }

    pub fn to_clearcoat_roughness_texture(&self) -> Option<&MaterialTexture> {
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

impl_state_value!(ClearcoatTextures);
