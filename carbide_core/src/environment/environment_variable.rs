use crate::Color;
use crate::prelude::EnvironmentColor;
use crate::prelude::EnvironmentFontSize;

#[derive(Debug, Clone)]
pub enum EnvironmentVariable {
    String {
        key: String,
        value: String,
    },
    U32 {
        key: String,
        value: u32,
    },
    F64 {
        key: String,
        value: f64,
    },
    Color {
        key: EnvironmentColor,
        value: Color,
    },
    FontSize {
        key: EnvironmentFontSize,
        value: u32,
    },
    I32 {
        key: String,
        value: i32,
    },
}
