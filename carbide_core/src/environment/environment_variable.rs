use crate::draw::Color;
use crate::environment::{EnvironmentColor, EnvironmentFontSize};

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
    I32 {
        key: String,
        value: i32,
    },
    Bool {
        key: &'static str,
        value: bool,
    },
    Color {
        key: String,
        value: Color,
    },
    FontSize {
        key: String,
        value: u32,
    },
    EnvironmentColor {
        key: EnvironmentColor,
        value: Color,
    },
    EnvironmentFontSize {
        key: EnvironmentFontSize,
        value: u32,
    },
}
