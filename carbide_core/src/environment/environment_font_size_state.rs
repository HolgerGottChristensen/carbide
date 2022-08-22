use std::ops::{Deref, DerefMut};

use crate::environment::{Environment, EnvironmentFontSize};
use crate::state::{ReadState, ValueRef, ValueRefMut, NewStateSync, State, StateKey};

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

impl NewStateSync for EnvironmentFontSizeState {
    fn sync(&mut self, env: &mut Environment) -> bool {
        if let Some(size) = env.get_font_size(&self.key) {
            if self.value != size {
                self.value = size;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl ReadState<u32> for EnvironmentFontSizeState {
    fn value(&self) -> ValueRef<u32> {
        ValueRef::Borrow(&self.value)
    }
}

impl State<u32> for EnvironmentFontSizeState {
    fn value_mut(&mut self) -> ValueRefMut<u32> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: u32) {
        self.value = value;
    }
}
