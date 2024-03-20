use std::any::Any;
use crate::draw::Color;
use crate::environment::{EnvironmentColor, EnvironmentFontSize};

#[allow(dead_code)]
pub(crate) enum EnvironmentVariable {
    EnvironmentColor {
        key: EnvironmentColor,
        value: Color,
    },
    EnvironmentFontSize {
        key: EnvironmentFontSize,
        value: u32,
    },
    Any {
        key: &'static str,
        value: Box<dyn Any>,
    }
}