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
}

impl Display for ImageLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageLoadError::NoAssetsFolder(path) => write!(f, "Couldn't load image: {}: no assets folder found", path.display()),
            ImageLoadError::Decode(path, error) => write!(f, "Couldn't load image: {}: {}", path.display(), error),
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

    Ok(true)
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
