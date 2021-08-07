use std::ops::{Deref, DerefMut};

use crate::Color;
use crate::prelude::{Environment, GlobalStateContract, State};
use crate::prelude::EnvironmentColor;
use crate::prelude::value_cell::ValueRef;
use crate::state::global_state::GlobalStateContainer;
use crate::state::state_key::StateKey;
use crate::state::value_cell::ValueRefMut;

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

impl State<Color> for EnvironmentColorState {
    fn capture_state(&mut self, env: &mut Environment) {
        if let Some(color) = env.get_color(&self.key) {
            self.value = color;
        }
    }

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<Color> {
        ValueRef::Borrow(&self.value)
    }

    fn value_mut(&mut self) -> ValueRefMut<Color> {
        ValueRefMut::Borrow(&mut self.value)
    }
}