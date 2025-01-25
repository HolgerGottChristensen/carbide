use std::any::Any;
use carbide_core::state::AnyReadState;
pub use environment_color::*;
pub use environment_font_size::EnvironmentFontSize;
pub use environment::{EnvironmentKey, EnvironmentKeyable, Environment};

use crate::widget::AnyWidget;

mod environment_color;
mod environment_font_size;
mod environment;