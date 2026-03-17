use std::ffi::OsStr;
use std::os::macos::raw::stat;
use crate::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use url::{ParseError, Url};
use carbide::widget::Image;

/// Unique image identifier.
///
/// Throughout carbide, images are referred to via their unique `Id`. By referring to images via
/// `Id`s, carbide can remain agnostic of the actual image or texture render used to represent each
/// image.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ImageId {
    None,
    Local(Arc<PathBuf>, ImageIdFormat),
    Remote(Arc<Url>, ImageIdFormat),
    InMemory(u32, ImageIdFormat)
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImageIdFormat {
    Unknown,
    Raster,
    Vector
}

impl ImageId {
    /// Generate a new image ID.
    pub fn new(into: impl IntoImageId) -> Self {
        into.into()
    }

    pub fn temp(format: ImageIdFormat) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let value = COUNTER.fetch_add(1, Ordering::Relaxed);
        ImageId::InMemory(value, format)
    }

    pub fn format(&self) -> ImageIdFormat {
        match self {
            ImageId::None => ImageIdFormat::Unknown,
            ImageId::Local(_, format) => *format,
            ImageId::Remote(_, format) => *format,
            ImageId::InMemory(_, format) => *format,
        }
    }
}

impl Default for ImageId {
    fn default() -> Self {
        ImageId::None
    }
}

// ---------------------------------------------------
//  Conversion implementations
// ---------------------------------------------------

impl ConvertIntoRead<ImageId> for &'static str {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&&'static str)-> ImageId, &'static str,  ImageId, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |c| {
            // TODO: Dont create new imageid and theirby parsing and creating arc every read.
            ImageId::new(*c)
        })
    }
}

impl ConvertIntoRead<ImageId> for PathBuf {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&PathBuf)-> ImageId, PathBuf, ImageId, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |c| {
            ImageId::new(c.clone())
        })
    }
}

pub trait IntoImageId {
    fn into(self) -> ImageId;
}

impl IntoImageId for &'static str {
    fn into(self) -> ImageId {
        match Url::parse(self) {
            Ok(url) if url.scheme() == "file" => {
                let format = match url.path().split('.').last() {
                    None => ImageIdFormat::Raster,
                    Some(ext) if ext == "svg" => ImageIdFormat::Vector,
                    Some(_) => ImageIdFormat::Raster,
                };

                ImageId::Local(
                    Arc::new(PathBuf::from_str(url.path()).unwrap()),
                    format
                )
            }
            Ok(url) => {
                let format = match url.path().split('.').last() {
                    None => ImageIdFormat::Raster,
                    Some(ext) if ext == "svg" => ImageIdFormat::Vector,
                    Some(_) => ImageIdFormat::Raster,
                };

                ImageId::Remote(Arc::new(url), format)
            }
            Err(_) => {
                let format = match self.split('.').last() {
                    None => ImageIdFormat::Raster,
                    Some(ext) if ext == "svg" => ImageIdFormat::Vector,
                    Some(_) => ImageIdFormat::Raster,
                };
                ImageId::Local(Arc::new(PathBuf::from_str(self).unwrap()), format)
            }
        }
    }
}

impl IntoImageId for PathBuf {
    fn into(self) -> ImageId {
        let format = match self.extension() {
            None => ImageIdFormat::Raster,
            Some(ext) if ext == "svg" => ImageIdFormat::Vector,
            Some(_) => ImageIdFormat::Raster,
        };

        ImageId::Local(Arc::new(self), format)
    }
}