use std::ops::{Deref, DerefMut};

use crate::prelude::{Environment, EnvironmentFontSize, GlobalStateContract, State};
use crate::prelude::global_state::GlobalStateContainer;
use crate::state::state_key::StateKey;

#[derive(Clone, Debug)]
pub struct EnvironmentFontSizeState {
    key: StateKey,
    value: u32,
}

impl EnvironmentFontSizeState {
    pub fn new(key: EnvironmentFontSize) -> Self {
        EnvironmentFontSizeState {
            key: StateKey::FontSize(key),
            value: 20,
        }
    }
}

impl Deref for EnvironmentFontSizeState {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for EnvironmentFontSizeState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<GS: GlobalStateContract> State<u32, GS> for EnvironmentFontSizeState {
    fn capture_state(&mut self, env: &mut Environment<GS>, _: &GlobalStateContainer<GS>) {
        if let Some(size) = env.get_font_size(&self.key) {
            self.value = size;
        }
    }

    fn release_state(&mut self, _: &mut Environment<GS>) {}
}