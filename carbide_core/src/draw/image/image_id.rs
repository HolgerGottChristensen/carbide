use std::sync::atomic::{AtomicU32, Ordering};

/// Unique image identifier.
///
/// Throughout carbide, images are referred to via their unique `Id`. By referring to images via
/// `Id`s, carbide can remain agnostic of the actual image or texture render used to represent each
/// image.
#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImageId(u32);

impl ImageId {
    /// Generate a new image ID.
    pub fn new() -> Self {
        static WIDGET_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
        ImageId(WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
