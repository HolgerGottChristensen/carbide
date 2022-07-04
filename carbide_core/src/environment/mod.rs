use crate::Color;
pub use environment::Environment;
pub use environment_color::EnvironmentColor;
pub use environment_color_state::EnvironmentColorState;
pub use environment_font_size::EnvironmentFontSize;
pub use environment_font_size_state::EnvironmentFontSizeState;
pub use environment_variable::EnvironmentVariable;

use crate::prelude::{F64State, I32State, StringState, U32State};
use crate::state::TState;
use crate::widget::Widget;

mod environment;
mod environment_color;
mod environment_color_state;
mod environment_font_size;
mod environment_font_size_state;
mod environment_variable;

#[derive(Debug, Clone)]
pub enum WidgetTransferAction {
    Push(Box<dyn Widget>),
    Pop,
    Replace(Box<dyn Widget>),
    PushVec(Vec<Box<dyn Widget>>),
    PopN(usize),
    PopAll,
    ReplaceAll(Box<dyn Widget>),
}

#[derive(Debug, Clone)]
pub enum EnvironmentStateContainer {
    String {
        key: String,
        value: StringState,
    },
    U32 {
        key: String,
        value: U32State,
    },
    F64 {
        key: String,
        value: F64State,
    },
    Color {
        key: EnvironmentColor,
        value: TState<Color>,
    },
    FontSize {
        key: EnvironmentFontSize,
        value: U32State,
    },
    I32 {
        key: String,
        value: I32State,
    },
}
