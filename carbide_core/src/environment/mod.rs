use std::any::Any;
use carbide_core::state::AnyReadState;
pub use environment::Environment;
pub use environment_color::*;
pub use environment_font_size::EnvironmentFontSize;
pub use environment_stack::{Key, Keyable, TypeMap as EnvironmentStack, AnyDebug};

use crate::widget::AnyWidget;

mod environment;
mod environment_color;
mod environment_font_size;
mod environment_stack;