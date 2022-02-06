use std::ops::{Deref, DerefMut};
use carbide_core::prelude::{NewStateSync, Listenable, Listener};

use crate::prelude::{Environment, EnvironmentFontSize, State};
use crate::state::{ReadState, SubscriberList, ValueRef, ValueRefMut};
use crate::state::StateKey;

#[derive(Clone, Debug)]
pub struct EnvironmentFontSizeState {
    key: StateKey,
    value: u32,
    subscribers: SubscriberList<u32>,
}

impl EnvironmentFontSizeState {
    pub fn new(key: EnvironmentFontSize) -> Self {
        EnvironmentFontSizeState {
            key: StateKey::FontSize(key),
            value: 20,
            subscribers: SubscriberList::new()
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
    fn sync(&mut self, env: &mut Environment) {
        if let Some(size) = env.get_font_size(&self.key) {
            if self.value != size {
                self.value = size;
                println!("Env font size changed to: {}", self.value);
                self.subscribers.notify(&size);
            }
        }
    }
}

impl Listenable<u32> for EnvironmentFontSizeState {
    fn subscribe(&self, subscriber: Box<dyn Listener<u32>>) {
        self.subscribers.add_subscriber(subscriber)
    }
}

impl ReadState<u32> for EnvironmentFontSizeState {
    fn value(&self) -> ValueRef<u32> {
        println!("{}", self.value);
        ValueRef::Borrow(&self.value)
    }
}

impl State<u32> for EnvironmentFontSizeState {
    fn value_mut(&mut self) -> ValueRefMut<u32> {
        ValueRefMut::Borrow(&mut self.value)
    }

    fn set_value(&mut self, value: u32) {
        self.value = value;
        self.notify();
    }

    fn notify(&self) {
        self.subscribers.notify(&self.value);
    }
}
