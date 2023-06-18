use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::state::{AnyReadState, IntoReadStateHelper, Map1, RMap1};

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

/*impl<T> IntoReadStateHelper<T, &'static str, Option<ImageId>> for T where T: AnyReadState<T=&'static str> + Clone {
    type Output = RMap1<fn(&&'static str)-> Option<ImageId>, &'static str,  Option<ImageId>, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map(self, |c| {
            Some(ImageId::new(c))
        })
    }
}

impl<T> IntoReadStateHelper<T, PathBuf, Option<ImageId>> for T where T: AnyReadState<T=PathBuf> + Clone {
    type Output = RMap1<fn(&PathBuf)-> Option<ImageId>, PathBuf,  Option<ImageId>, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map(self, |c| {
            Some(ImageId::new(c))
        })
    }
}*/