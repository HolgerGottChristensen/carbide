use std::ops::{Deref, DerefMut};

use crate::environment::{Environment, EnvironmentFontSize};
use crate::state::{NewStateSync, ReadState, State, StateKey, ValueRef, ValueRefMut};
use crate::text::FontSize;

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

impl ReadState for EnvironmentFontSizeState {
    type T = FontSize;
    fn value(&self) -> ValueRef<FontSize> {
        ValueRef::Borrow(&self.value)
    }
}

impl State for EnvironmentFontSizeState {
    fn value_mut(&mut self) -> ValueRefMut<FontSize> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: FontSize) {
        self.value = value;
    }
}
