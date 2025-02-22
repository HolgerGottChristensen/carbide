use crate::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Unique image identifier.
///
/// Throughout carbide, images are referred to via their unique `Id`. By referring to images via
/// `Id`s, carbide can remain agnostic of the actual image or texture render used to represent each
/// image.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageId(Arc<PathBuf>);

impl ImageId {
    /// Generate a new image ID.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        ImageId(Arc::new(path.into()))
    }

    pub fn is_relative(&self) -> bool {
        self.0.is_relative()
    }

    pub fn file_stem(&self) -> Option<String> {
        Some(self.0.file_stem().unwrap_or(self.0.as_os_str()).to_str()?.to_string())
    }
}

impl Default for ImageId {
    fn default() -> Self {
        ImageId(Arc::new(PathBuf::default()))
    }
}

impl AsRef<Path> for ImageId {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

// ---------------------------------------------------
//  Conversion implementations
// ---------------------------------------------------

impl ConvertIntoRead<Option<ImageId>> for &'static str {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&&'static str)-> Option<ImageId>, &'static str,  Option<ImageId>, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |c| {
            Some(ImageId::new(c))
        })
    }
}

impl ConvertIntoRead<Option<ImageId>> for PathBuf {
    type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&PathBuf)-> Option<ImageId>, PathBuf,  Option<ImageId>, G>;

    fn convert<F: AnyReadState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::read_map(f, |c| {
            Some(ImageId::new(c))
        })
    }
}