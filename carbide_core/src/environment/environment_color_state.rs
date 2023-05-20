use std::ops::{Deref, DerefMut};

use crate::Color;
use crate::environment::{Environment, EnvironmentColor};
use crate::render::Style;
use crate::state::{NewStateSync, ReadState, State, StateKey, ValueRef, ValueRefMut};

#[derive(Clone, Debug)]
pub struct EnvironmentColorState {
    key: StateKey,
    value: Style,
}

impl EnvironmentColorState {
    pub fn new(key: EnvironmentColor) -> Self {
        EnvironmentColorState {
            key: StateKey::Color(key),
            value: Style::Color(Color::Rgba(0.0, 0.0, 0.0, 1.0)),
        }
    }
}

impl Deref for EnvironmentColorState {
    type Target = Style;

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
            if self.value != Style::Color(color) {
                self.value = Style::Color(color);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl ReadState for EnvironmentColorState {
    type T = Style;
    fn value(&self) -> ValueRef<Style> {
        ValueRef::Borrow(&self.value)
    }
}

impl State for EnvironmentColorState {
    fn value_mut(&mut self) -> ValueRefMut<Style> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: Style) {
        self.value = value;
    }
}
