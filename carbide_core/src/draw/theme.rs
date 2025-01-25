use crate::environment::EnvironmentKey;
use crate::impl_state_value;

#[derive(Copy, Clone, Debug)]
pub enum Theme {
    Light,
    Dark
}

impl EnvironmentKey for Theme {
    type Value = Theme;
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

impl_state_value!(Theme);