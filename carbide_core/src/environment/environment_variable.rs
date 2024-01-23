use std::any::Any;
use std::fmt::Debug;
use dyn_clone::DynClone;
use crate::draw::Color;
use crate::environment::{EnvironmentColor, EnvironmentFontSize};

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