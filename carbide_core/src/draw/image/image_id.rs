use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// Unique image identifier.
///
/// Throughout carbide, images are referred to via their unique `Id`. By referring to images via
/// `Id`s, carbide can remain agnostic of the actual image or texture render used to represent each
/// image.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageId(PathBuf);

impl ImageId {
    /// Generate a new image ID.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        ImageId(path.into())
    }
}

impl Default for ImageId {
    fn default() -> Self {
        ImageId(PathBuf::default())
    }
}

impl AsRef<Path> for ImageId {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}
