use std::ops::{Deref, DerefMut};

use crate::prelude::{Environment, EnvironmentFontSize, State};
use crate::state::{ValueRef, ValueRefMut};
use crate::state::StateKey;

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

impl State<u32> for EnvironmentFontSizeState {
    fn capture_state(&mut self, env: &mut Environment) {
        if let Some(size) = env.get_font_size(&self.key) {
            self.value = size;
        }
    }

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<u32> {
        todo!()//Box::new(self.value.borrow())
    }

    fn value_mut(&mut self) -> ValueRefMut<u32> {
        todo!()//Box::new(self.value.borrow_mut())
    }
}