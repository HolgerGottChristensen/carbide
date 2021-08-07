pub use environment::Environment;
pub use environment_color::EnvironmentColor;
pub use environment_color_state::EnvironmentColorState;
pub use environment_font_size::EnvironmentFontSize;
pub use environment_font_size_state::EnvironmentFontSizeState;
pub use environment_variable::EnvironmentVariable;

use crate::prelude::{ColorState, F64State, I32State, StringState, U32State};

mod environment;
mod environment_color;
mod environment_font_size;
mod environment_variable;
mod environment_color_state;
mod environment_font_size_state;

#[derive(Debug, Clone)]
pub enum EnvironmentStateContainer {
    String { key: String, value: StringState },
    U32 { key: String, value: U32State },
    F64 { key: String, value: F64State },
    Color { key: EnvironmentColor, value: ColorState },
    FontSize { key: EnvironmentFontSize, value: U32State },
    I32 { key: String, value: I32State },
}

