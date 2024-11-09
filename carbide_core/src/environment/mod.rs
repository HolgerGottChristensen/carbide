use std::any::Any;
use carbide_core::state::AnyReadState;
pub use environment::Environment;
pub use environment_color::*;
pub use environment_font_size::EnvironmentFontSize;
pub use environment_stack::{Key, Keyable, TypeMap as EnvironmentStack};
pub(crate) use environment_variable::EnvironmentVariable;

use crate::widget::AnyWidget;

mod environment;
mod environment_color;
mod environment_font_size;
mod environment_variable;
pub mod environment_state_key;
mod environment_stack;

#[derive(Debug, Clone)]
pub enum WidgetTransferAction {
    Push(Box<dyn AnyWidget>),
    Pop,
    Replace(Box<dyn AnyWidget>),
    PushVec(Vec<Box<dyn AnyWidget>>),
    PopN(usize),
    PopAll,
    ReplaceAll(Box<dyn AnyWidget>),
}