use std::ops::{Deref, DerefMut};

use crate::prelude::EnvironmentColor;
use crate::prelude::{Environment, State};
use crate::state::StateKey;
use crate::state::{NewStateSync, ReadState, ValueRef, ValueRefMut};
use crate::Color;

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

impl NewStateSync for EnvironmentColorState {
    fn sync(&mut self, env: &mut Environment) -> bool {
        if let Some(color) = env.get_color(&self.key) {
            if self.value != color {
                self.value = color;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl ReadState<Color> for EnvironmentColorState {
    fn value(&self) -> ValueRef<Color> {
        ValueRef::Borrow(&self.value)
    }
}

impl State<Color> for EnvironmentColorState {
    fn value_mut(&mut self) -> ValueRefMut<Color> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: Color) {
        self.value = value;
    }
}
