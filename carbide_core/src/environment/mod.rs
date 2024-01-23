use std::any::Any;
use carbide_core::state::AnyReadState;
pub use environment::Environment;
pub use environment_color::*;
pub use environment_font_size::EnvironmentFontSize;
pub(crate) use environment_variable::EnvironmentVariable;

use crate::draw::Color;
use crate::state::TState;
use crate::widget::AnyWidget;

mod environment;
mod environment_color;
mod environment_font_size;
mod environment_variable;
pub mod environment_state_key;

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

pub enum EnvironmentStateContainer {
    String {
        key: String,
        value: TState<String>,
    },
    U32 {
        key: String,
        value: TState<u32>,
    },
    F64 {
        key: String,
        value: TState<f64>,
    },
    Color {
        key: EnvironmentColor,
        value: TState<Color>,
    },
    FontSize {
        key: EnvironmentFontSize,
        value: TState<u32>,
    },
    I32 {
        key: String,
        value: TState<i32>,
    },
    Bool {
        key: &'static str,
        value: Box<dyn AnyReadState<T=bool>>,
    },
    Any {
        key: &'static str,
        value: Box<dyn AnyReadState<T=Box<dyn Any>>>,
    }
}
