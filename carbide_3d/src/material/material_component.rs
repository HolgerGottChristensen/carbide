use carbide::draw::ImageId;

/// Generic container for a component of a material that could either be from a
/// texture or a fixed value.
#[derive(Debug, Clone)]
pub enum MaterialComponent<T> {
    None,
    Value(T),
    Texture(ImageId),
    TextureValue { texture: ImageId, value: T },
}

impl<T> Default for MaterialComponent<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T: Copy> MaterialComponent<T> {
    pub fn to_value(&self, default: T) -> T {
        match *self {
            Self::Value(value) | Self::TextureValue { value, .. } => value,
            Self::None | Self::Texture(_) => default,
        }
    }

    pub fn is_texture(&self) -> bool {
        matches!(*self, Self::Texture(..) | Self::TextureValue { .. })
    }

    pub fn to_texture(&self) -> Option<&ImageId> {
        match *self {
            Self::None | Self::Value(_) => None,
            Self::Texture(ref texture) | Self::TextureValue { ref texture, .. } => Some(texture),
        }
    }
}