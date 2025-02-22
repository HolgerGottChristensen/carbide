use crate::material::material_flags::MaterialFlags;
use bitflags::Flags;
use carbide::color::WHITE;
use carbide::draw::{Color, ImageId};
use carbide::impl_state_value;
use carbide::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};

/// How the albedo color should be determined.
#[derive(Debug, Clone)]
pub enum AlbedoComponent {
    /// No albedo color.
    None,
    /// Albedo color is the vertex value.
    Vertex {
        /// Vertex should be converted from srgb -> linear before
        /// multiplication.
        srgb: bool,
    },
    /// Albedo color is the given value.
    Value(Color),
    /// Albedo color is the given value multiplied by the vertex color.
    ValueVertex {
        value: Color,
        /// Vertex should be converted from srgb -> linear before
        /// multiplication.
        srgb: bool,
    },
    /// Albedo color is loaded from the given texture.
    Texture(ImageId),
    /// Albedo color is loaded from the given texture, then multiplied
    /// by the vertex color.
    TextureVertex {
        texture: ImageId,
        /// Vertex should be converted from srgb -> linear before
        /// multiplication.
        srgb: bool,
    },
    /// Albedo color is loaded from given texture, then multiplied
    /// by the given value.
    TextureValue { texture: ImageId, value: Color },
    /// Albedo color is loaded from the given texture, then multiplied
    /// by the vertex color and the given value.
    TextureVertexValue {
        texture: ImageId,
        /// Vertex should be converted from srgb -> linear before
        /// multiplication.
        srgb: bool,
        value: Color,
    },
}

impl Default for AlbedoComponent {
    fn default() -> Self {
        Self::None
    }
}

impl AlbedoComponent {
    pub fn to_value(&self) -> Color {
        match *self {
            Self::Value(value) => value,
            Self::ValueVertex { value, .. } => value,
            Self::TextureValue { value, .. } => value,
            _ => WHITE,
        }
    }

    pub fn to_flags(&self) -> MaterialFlags {
        match *self {
            Self::None => MaterialFlags::empty(),
            Self::Value(_) | Self::Texture(_) | Self::TextureValue { .. } => MaterialFlags::ALBEDO_ACTIVE,
            Self::Vertex { srgb: false }
            | Self::ValueVertex { srgb: false, .. }
            | Self::TextureVertex { srgb: false, .. }
            | Self::TextureVertexValue { srgb: false, .. } => {
                MaterialFlags::ALBEDO_ACTIVE | MaterialFlags::ALBEDO_BLEND
            }
            Self::Vertex { srgb: true }
            | Self::ValueVertex { srgb: true, .. }
            | Self::TextureVertex { srgb: true, .. }
            | Self::TextureVertexValue { srgb: true, .. } => {
                MaterialFlags::ALBEDO_ACTIVE | MaterialFlags::ALBEDO_BLEND | MaterialFlags::ALBEDO_VERTEX_SRGB
            }
        }
    }

    pub fn is_texture(&self) -> bool {
        matches!(
            *self,
            Self::Texture(..)
                | Self::TextureVertex { .. }
                | Self::TextureValue { .. }
                | Self::TextureVertexValue { .. }
        )
    }

    pub fn to_texture(&self) -> Option<&ImageId> {
        match *self {
            Self::None | Self::Vertex { .. } | Self::Value(_) | Self::ValueVertex { .. } => None,
            Self::Texture(ref texture)
            | Self::TextureVertex { ref texture, .. }
            | Self::TextureValue { ref texture, .. }
            | Self::TextureVertexValue { ref texture, .. } => Some(texture),
        }
    }
}

impl_state_value!(AlbedoComponent);

impl ConvertIntoRead<AlbedoComponent> for Color {
    type Output<G: AnyReadState<T=Color> + Clone> = RMap1<fn(&Color)->AlbedoComponent, Color, AlbedoComponent, G>;

    fn convert<F: AnyReadState<T=Color> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |color| {
            AlbedoComponent::Value(*color)
        })
    }
}


impl ConvertIntoRead<AlbedoComponent> for ImageId {
    type Output<G: AnyReadState<T=ImageId> + Clone> = RMap1<fn(&ImageId)->AlbedoComponent, ImageId, AlbedoComponent, G>;

    fn convert<F: AnyReadState<T=ImageId> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |id| {
            AlbedoComponent::Texture(id.clone())
        })
    }
}