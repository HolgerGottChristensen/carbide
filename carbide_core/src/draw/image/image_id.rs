use crate::draw::image::image_format::ImageFormat;
use crate::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use url::Url;

/// Unique image identifier.
///
/// Throughout carbide, images are referred to via their unique `Id`. By referring to images via
/// `Id`s, carbide can remain agnostic of the actual image or texture render used to represent each
/// image.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ImageId {
    None,
    System(String, ImageFormat),
    Local(Arc<PathBuf>, ImageFormat),
    Remote(Arc<Url>, ImageFormat),
    InMemory(u32, ImageFormat)
}

impl ImageId {
    /// Generate a new image ID.
    pub fn new(into: impl IntoImageId) -> Self {
        into.into()
    }

    pub fn system(name: String, format: ImageFormat) -> Self {
        ImageId::System(name, format)
    }

    pub fn temp(format: ImageFormat) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let value = COUNTER.fetch_add(1, Ordering::Relaxed);
        ImageId::InMemory(value, format)
    }

    pub fn format(&self) -> ImageFormat {
        match self {
            ImageId::None => ImageFormat::Unknown,
            ImageId::Local(_, format) => *format,
            ImageId::Remote(_, format) => *format,
            ImageId::InMemory(_, format) => *format,
            ImageId::System(_, format) => *format,
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
                    None => ImageFormat::Unknown,
                    Some(ext) => ImageFormat::from_extension(ext),
                };

                ImageId::Local(
                    Arc::new(PathBuf::from_str(url.path()).unwrap()),
                    format
                )
            }
            Ok(url) => {
                let format = match url.path().split('.').last() {
                    None => ImageFormat::Unknown,
                    Some(ext) => ImageFormat::from_extension(ext),
                };

                ImageId::Remote(Arc::new(url), format)
            }
            Err(_) => {
                let format = match self.split('.').last() {
                    None => ImageFormat::Unknown,
                    Some(ext) => ImageFormat::from_extension(ext),
                };
                ImageId::Local(Arc::new(PathBuf::from_str(self).unwrap()), format)
            }
        }
    }
}

impl IntoImageId for PathBuf {
    fn into(self) -> ImageId {
        let format = match self.extension() {
            None => ImageFormat::Unknown,
            Some(ext) => ImageFormat::from_extension(ext.to_str().unwrap()),
        };

        ImageId::Local(Arc::new(self), format)
    }
}