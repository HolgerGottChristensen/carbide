use carbide::draw::ImageId;
use carbide::impl_state_value;
use carbide::math::{Matrix3, Rad, SquareMatrix, Vector2};

/// An image used as a material's map, with how the mesh's texture coordinates reach it: which of
/// the mesh's two coordinate sets, and a 2D transform of them, as glTF's `textureInfo` with
/// `KHR_texture_transform`, or three.js's `Texture.offset` and `repeat`.
///
/// The material's own [`uv_transform`](crate::material::pbr_material::PbrMaterial::uv_transform)
/// (or [`tiling`](crate::material::pbr_material::PbrMaterial::tiling)) applies after the map's,
/// to every map. An `ImageId` converts to a map with the identity transform on the first set.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialTexture {
    /// The image.
    pub image: ImageId,
    /// Transforms the texture coordinates before the image is sampled: a 2D affine transform as a
    /// 3×3 matrix. The identity by default.
    pub transform: Matrix3<f32>,
    /// Which of the mesh's texture coordinates are read: 0 (`texture_coords_0`, the default) or 1
    /// (`texture_coords_1`). Other values read 0.
    pub uv_set: u8,
}

impl MaterialTexture {
    /// `image`, with the identity transform, on the first coordinate set.
    pub fn new(image: impl Into<ImageId>) -> MaterialTexture {
        MaterialTexture { image: image.into(), transform: Matrix3::identity(), uv_set: 0 }
    }

    /// The map with its texture coordinates transformed by `transform`.
    pub fn transform(mut self, transform: Matrix3<f32>) -> MaterialTexture {
        self.transform = transform;
        self
    }

    /// The map with its texture coordinates scaled by `scale`, rotated by `rotation` and then
    /// offset by `offset`, in that order, as glTF's `KHR_texture_transform`
    /// (`translation · rotation · scale`). A positive rotation turns the image clockwise on the
    /// surface, as Khronos's sample viewer, three.js and Bevy show it.
    pub fn offset_rotation_scale(self, offset: Vector2<f32>, rotation: Rad<f32>, scale: Vector2<f32>) -> MaterialTexture {
        let (sin, cos) = (rotation.0.sin(), rotation.0.cos());
        // Column by column. The rotation is u' = cos·u + sin·v, v' = −sin·u + cos·v, as three.js's
        // `Matrix3.setUvTransform` and Bevy's loader (which negates the angle) build it, and as
        // TextureTransformTest's screenshot shows. The extension README's GLSL, read as GLSL fills
        // a matrix, is its transpose, which turns the image the other way.
        let translation = Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, offset.x, offset.y, 1.0);
        let rotation = Matrix3::new(cos, -sin, 0.0, sin, cos, 0.0, 0.0, 0.0, 1.0);
        let scale = Matrix3::new(scale.x, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 1.0);
        self.transform(translation * rotation * scale)
    }

    /// The map repeated `tiling.x` times across and `tiling.y` times down.
    pub fn tiling(self, tiling: Vector2<f32>) -> MaterialTexture {
        self.transform(Matrix3::from_nonuniform_scale(tiling.x, tiling.y))
    }

    /// The map read with the mesh's coordinate set `uv_set`, 0 or 1.
    pub fn uv_set(mut self, uv_set: u8) -> MaterialTexture {
        self.uv_set = uv_set;
        self
    }
}

impl From<ImageId> for MaterialTexture {
    fn from(image: ImageId) -> Self {
        MaterialTexture::new(image)
    }
}

impl From<&ImageId> for MaterialTexture {
    fn from(image: &ImageId) -> Self {
        MaterialTexture::new(image.clone())
    }
}

impl_state_value!(MaterialTexture);

#[cfg(test)]
mod tests {
    use super::*;
    use carbide::math::{InnerSpace, Vector3};

    /// Scale, then rotation, then offset: with offset (0, 1), rotation 90° and scale (0.5, 0.5),
    /// (0, 0) moves to (0, 1), and (1, 0) to (0.5, 0) scaled, (0, −0.5) turned, and (0, 0.5)
    /// offset. And as three.js's `setUvTransform`: u' = cos·u + sin·v, v' = −sin·u + cos·v.
    #[test]
    fn offset_rotation_scale_matches_gltf() {
        let map = MaterialTexture::new(ImageId::default())
            .offset_rotation_scale(Vector2::new(0.0, 1.0), Rad(std::f32::consts::FRAC_PI_2), Vector2::new(0.5, 0.5));
        let at = |u: f32, v: f32| map.transform * Vector3::new(u, v, 1.0);
        assert!((at(0.0, 0.0) - Vector3::new(0.0, 1.0, 1.0)).magnitude() < 1e-5, "{:?}", at(0.0, 0.0));
        assert!((at(1.0, 0.0) - Vector3::new(0.0, 0.5, 1.0)).magnitude() < 1e-5, "{:?}", at(1.0, 0.0));

        // 30°, no scale or offset: three.js's matrix, row by row.
        let (sin, cos) = (0.5f32, 3f32.sqrt() / 2.0);
        let turned = MaterialTexture::new(ImageId::default())
            .offset_rotation_scale(Vector2::new(0.0, 0.0), Rad(std::f32::consts::FRAC_PI_6), Vector2::new(1.0, 1.0));
        let uv = turned.transform * Vector3::new(0.3, 0.7, 1.0);
        assert!((uv.x - (cos * 0.3 + sin * 0.7)).abs() < 1e-5 && (uv.y - (-sin * 0.3 + cos * 0.7)).abs() < 1e-5, "{uv:?}");
    }
}
