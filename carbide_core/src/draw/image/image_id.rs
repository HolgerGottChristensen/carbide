use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::state::{TState, ValueState};

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

impl Into<TState<Option<ImageId>>> for &str {
    fn into(self) -> TState<Option<ImageId>> {
        ValueState::new(Some(ImageId::new(self)))
    }
}

impl Into<TState<Option<ImageId>>> for PathBuf {
    fn into(self) -> TState<Option<ImageId>> {
        ValueState::new(Some(ImageId::new(self)))
    }
}
