mod scene;
mod scene_manager;
mod scene_sequence;

pub use scene::*;
pub use scene_manager::*;
pub use scene_sequence::*;
use crate::widget::WidgetId;

pub type SceneId = WidgetId;