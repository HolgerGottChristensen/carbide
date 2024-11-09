mod scene;
mod scene_manager;

use std::sync::atomic::{AtomicUsize, Ordering};
pub use scene::*;
pub use scene_manager::*;


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneId(pub usize);

impl SceneId {
    pub fn new() -> SceneId {
        static SCENE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        SceneId(SCENE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}