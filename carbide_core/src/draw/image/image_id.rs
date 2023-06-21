use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::state::{AnyReadState, ConvertIntoRead, Map1, RMap1};

/// Unique image identifier.
///
/// Throughout carbide, images are referred to via their unique `Id`. By referring to images via
/// `Id`s, carbide can remain agnostic of the actual image or texture render used to represent each
/// image.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageId(Rc<PathBuf>);

impl ImageId {
    /// Generate a new image ID.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        ImageId(Rc::new(path.into()))
    }

    pub fn is_relative(&self) -> bool {
        self.0.is_relative()
    }
}

impl Default for ImageId {
    fn default() -> Self {
        ImageId(Rc::new(PathBuf::default()))
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