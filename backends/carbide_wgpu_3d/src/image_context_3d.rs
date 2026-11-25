use carbide_3d::InnerImageContext3d;
use carbide_core::draw::{ImageId, Texture};
use carbide_core::environment::Environment;

/// The image context of the wgpu 3D renderer.
///
/// Textures on materials are not supported yet, and the shaders don't sample them: this context
/// stores no textures, so every texture is missing and updates are rejected. Loading them is the
/// TODO item "Textures through the image context" in `carbide_3d`'s design document.
#[derive(Debug, Clone)]
pub struct ImageContext3d;

impl InnerImageContext3d for ImageContext3d {
    fn texture_exist(&self, _id: &ImageId, _env: &mut Environment) -> bool {
        false
    }

    fn texture_dimensions(&self, _id: &ImageId, _env: &mut Environment) -> Option<(u32, u32)> {
        None
    }

    fn update_texture(&mut self, _id: ImageId, _texture: Texture, _env: &mut Environment) -> bool {
        false
    }
}
