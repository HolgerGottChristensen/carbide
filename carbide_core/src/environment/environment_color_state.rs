use std::ops::{Deref, DerefMut};

use crate::Color;
use crate::prelude::{Environment, GlobalStateContract, State};
use crate::prelude::EnvironmentColor;
use crate::state::global_state::GlobalStateContainer;
use crate::state::state_key::StateKey;

#[derive(Clone, Debug)]
pub struct EnvironmentColorState {
    key: StateKey,
    value: Color,
}

impl EnvironmentColorState {
    pub fn new(key: EnvironmentColor) -> Self {
        EnvironmentColorState {
            key: StateKey::Color(key),
            value: Color::Rgba(0.0, 0.0, 0.0, 1.0),
        }
    }
}

impl Deref for EnvironmentColorState {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for EnvironmentColorState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<GS: GlobalStateContract> State<Color, GS> for EnvironmentColorState {
    fn capture_state(&mut self, env: &mut Environment<GS>, _: &GlobalStateContainer<GS>) {
        if let Some(color) = env.get_color(&self.key) {
            self.value = color;
        }
    }

    fn release_state(&mut self, _: &mut Environment<GS>) {}
}