use crate::material::material_flags::MaterialFlags;
use crate::material::normal_texture_y_direction::NormalTextureYDirection;
use carbide::draw::ImageId;
use crate::material::material_texture::MaterialTexture;
use carbide::impl_state_value;
use carbide::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};

/// How normals should be derived
#[derive(Debug, Clone)]
#[derive(Default)]
pub enum NormalTexture {
    /// No normal texture.
    #[default]
    None,
    /// Normal stored in RGB values.
    Tricomponent(MaterialTexture, NormalTextureYDirection),
    /// Normal stored in RG values, third value should be reconstructed.
    Bicomponent(MaterialTexture, NormalTextureYDirection),
    /// Normal stored in Green and Alpha values, third value should be
    /// reconstructed. This is useful for storing in BC3 or BC7 compressed
    /// textures.
    BicomponentSwizzled(MaterialTexture, NormalTextureYDirection),
}

impl NormalTexture {
    pub fn to_texture(&self) -> Option<&MaterialTexture> {
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

impl_state_value!(NormalTexture);

impl ConvertIntoRead<NormalTexture> for ImageId {
    type Output<G: AnyReadState<T=ImageId> + Clone> = RMap1<fn(&ImageId)->NormalTexture, ImageId, NormalTexture, G>;

    fn convert<F: AnyReadState<T=ImageId> + Clone>(f: F) -> Self::Output<F> {
        // Green up (OpenGL), as glTF's normal maps are, and the type's default.
        Map1::read_map(f, |id| {
            NormalTexture::Tricomponent(MaterialTexture::new(id.clone()), NormalTextureYDirection::Up)
        })
    }
}
impl ConvertIntoRead<NormalTexture> for MaterialTexture {
    type Output<G: AnyReadState<T=MaterialTexture> + Clone> = RMap1<fn(&MaterialTexture)->NormalTexture, MaterialTexture, NormalTexture, G>;

    fn convert<F: AnyReadState<T=MaterialTexture> + Clone>(f: F) -> Self::Output<F> {
        // Green up (OpenGL), as an `ImageId`'s.
        Map1::read_map(f, |texture| {
            NormalTexture::Tricomponent(texture.clone(), NormalTextureYDirection::Up)
        })
    }
}
