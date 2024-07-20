use std::any::Any;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct LayerId(u32);

impl LayerId {
    /// Generate a new layer ID.
    pub fn new() -> Self {
        static WIDGET_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
        LayerId(WIDGET_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for LayerId {
    fn default() -> Self {
        LayerId::new()
    }
}

pub trait InnerLayer: Any {
    fn dimensions(&self) -> (u32, u32);
}

pub struct NoopLayer;

impl InnerLayer for NoopLayer {
    fn dimensions(&self) -> (u32, u32) {
        (0, 0)
    }
}

// Waiting for trait upcasting to be implemented: https://github.com/rust-lang/rust/issues/65991
pub struct Layer<'a> {
    pub inner: &'a dyn Any,
    pub inner2: &'a dyn InnerLayer,
}

impl<'a> Layer<'a> {
    pub fn dimensions(&self) -> (u32, u32) {
        self.inner2.dimensions()
    }
}
