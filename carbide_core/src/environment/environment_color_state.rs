use std::ops::{Deref, DerefMut};
use carbide_core::state::AnyReadState;

use crate::Color;
use crate::environment::{Environment, EnvironmentColor};
use crate::render::Style;
use crate::state::{AnyState, NewStateSync, ReadState, State, StateKey, ValueRef, ValueRefMut};

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

/*impl Deref for EnvironmentColorState {
    type Target = Style;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for EnvironmentColorState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}*/

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

impl AnyReadState for EnvironmentColorState {
    type T = Color;
    fn value_dyn(&self) -> ValueRef<Color> {
        ValueRef::Borrow(&self.value)
    }
}

impl AnyState for EnvironmentColorState {
    fn value_dyn_mut(&mut self) -> ValueRefMut<Color> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value_dyn(&mut self, value: Color) {
        self.value = value;
    }
}
