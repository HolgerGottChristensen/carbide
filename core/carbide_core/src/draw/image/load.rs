use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use crate::draw::image::pre_multiply::pre_multiply_rgba8;
use crate::draw::{ImageContext, ImageFormat, ImageId, Texture, TextureFormat};
use crate::environment::Environment;

/// Why a raster image could not be loaded.
#[derive(Debug)]
pub enum ImageLoadError {
    /// No `assets` folder was found to resolve the relative path against.
    NoAssetsFolder(PathBuf),
    /// The file could not be opened or decoded.
    Decode(PathBuf, image::ImageError),
    /// Encoded bytes in memory could not be decoded.
    DecodeBytes(ImageId, image::ImageError),
}

impl Display for ImageLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageLoadError::NoAssetsFolder(path) => write!(f, "Couldn't load image: {}: no assets folder found", path.display()),
            ImageLoadError::Decode(path, error) => write!(f, "Couldn't load image: {}: {}", path.display(), error),
            ImageLoadError::DecodeBytes(id, error) => write!(f, "Couldn't decode image {:?}: {}", id, error),
        }
    }
}

impl std::error::Error for ImageLoadError {}

/// Load the local raster image `id` into `image`, unless it is there already.
///
/// Relative paths are resolved against the nearest `assets` folder. The image is decoded, its
/// alpha premultiplied, and uploaded as RGBA8 with `update_texture`. Ids that are not local raster
/// images (vector images, GIFs, remote and in-memory images, `ImageId::None`) are left alone and
/// return `Ok(false)`. Returns `Ok(true)` when the image was loaded by this call.
pub fn load_local_raster(id: &ImageId, image: &mut dyn ImageContext, env: &mut Environment) -> Result<bool, ImageLoadError> {
    let ImageId::Local(path, format) = id else {
        return Ok(false);
    };

    if matches!(format, ImageFormat::Svg | ImageFormat::Gif) || image.exist(id, env) {
        return Ok(false);
    }

    let full_path = resolve_asset_path(path)?;

    let decoded = image::open(&full_path)
        .map_err(|error| ImageLoadError::Decode(full_path.clone(), error))?;

    upload_decoded(id, decoded, image, env);
    Ok(true)
}

/// Decode `bytes`, an encoded raster image (PNG, JPEG or another format the `image` crate reads,
/// told apart by its contents), into `image` under `id`, unless `id` is there already: for images
/// that come as bytes, such as those embedded in a glTF file. Decoded, premultiplied and uploaded
/// as [`load_local_raster`] does. Returns `Ok(true)` when the image was loaded by this call.
pub fn load_encoded_raster(id: &ImageId, bytes: &[u8], image: &mut dyn ImageContext, env: &mut Environment) -> Result<bool, ImageLoadError> {
    if image.exist(id, env) {
        return Ok(false);
    }

    let decoded = image::load_from_memory(bytes)
        .map_err(|error| ImageLoadError::DecodeBytes(id.clone(), error))?;

    upload_decoded(id, decoded, image, env);
    Ok(true)
}

/// Upload `decoded` under `id`: float images (`.hdr`, `.exr` and other 32-bit float images) as
/// linear RGBA16F, as they are; others as premultiplied RGBA8.
fn upload_decoded(id: &ImageId, decoded: image::DynamicImage, image: &mut dyn ImageContext, env: &mut Environment) {
    if matches!(decoded.color(), image::ColorType::Rgb32F | image::ColorType::Rgba32F) {
        upload_float(id, decoded, image, env);
        return;
    }

    // One conversion to RGBA8, with no copy when the image already is. Alpha is premultiplied in
    // place, and only when the image has alpha: without it, every alpha is 1 and premultiplying
    // changes nothing.
    let has_alpha = decoded.color().has_alpha();
    let (width, height) = (decoded.width(), decoded.height());
    let mut pixels = decoded.into_rgba8().into_raw();
    if has_alpha {
        pre_multiply_rgba8(&mut pixels);
    }

    let texture = Texture {
        width,
        height,
        bytes_per_row: width * 4,
        format: TextureFormat::RGBA8,
        data: &pixels,
    };

    image.update_texture(id, texture, env);
}

/// `value`, clamped to what a 16-bit float holds as a finite number; 0 for NaN.
fn to_finite_f16(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-half::f16::MAX.to_f32(), half::f16::MAX.to_f32())
    }
}

/// Upload `decoded`, a float image, as linear RGBA16F: values above 1 are kept, and alpha isn't
/// premultiplied (HDR images usually have none). Values are clamped to the largest finite 16-bit
/// float, 65504, and NaN becomes 0: a sun in an HDR panorama can be brighter, and an infinite
/// texel turns into NaN wherever it is filtered (0 × ∞), blacking out whole mip levels.
fn upload_float(id: &ImageId, decoded: image::DynamicImage, image: &mut dyn ImageContext, env: &mut Environment) {
    let (width, height) = (decoded.width(), decoded.height());
    let pixels: Vec<u8> = decoded.into_rgba32f().into_raw().into_iter()
        .flat_map(|value| half::f16::from_f32(to_finite_f16(value)).to_le_bytes())
        .collect();

    let texture = Texture {
        width,
        height,
        bytes_per_row: width * 8,
        format: TextureFormat::RGBA16F,
        data: &pixels,
    };

    image.update_texture(id, texture, env);
}

/// Resolve `path` against the nearest `assets` folder, unless it is absolute.
pub fn resolve_asset_path(path: &Path) -> Result<PathBuf, ImageLoadError> {
    if path.is_relative() {
        let assets = crate::locate_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets")
            .map_err(|_| ImageLoadError::NoAssetsFolder(path.to_path_buf()))?;

        Ok(assets.join(path))
    } else {
        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::to_finite_f16;

    #[test]
    fn floats_beyond_half_precision_are_clamped() {
        assert_eq!(half::f16::from_f32(to_finite_f16(103_424.0)), half::f16::MAX);
        assert_eq!(half::f16::from_f32(to_finite_f16(f32::INFINITY)), half::f16::MAX);
        assert_eq!(half::f16::from_f32(to_finite_f16(-1e9)), half::f16::MIN);
        assert_eq!(to_finite_f16(f32::NAN), 0.0);
        assert_eq!(to_finite_f16(2.5), 2.5);
    }
}
