use crate::environment::{EnvironmentKey, Shared};
use crate::impl_state_value;

#[derive(Copy, Clone, Debug)]
#[derive(Default)]
pub enum Theme {
    Light,
    #[default]
    Dark
}

impl EnvironmentKey for Theme {
    type Value = Theme;
    type Access = Shared;
}


impl_state_value!(Theme);