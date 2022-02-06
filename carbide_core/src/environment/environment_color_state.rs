use std::ops::{Deref, DerefMut};
use carbide_core::prelude::Listener;

use crate::Color;
use crate::prelude::{Environment, State};
use crate::prelude::EnvironmentColor;
use crate::state::{Listenable, ReadState, SubscriberList, ValueRef, ValueRefMut};
use crate::state::StateKey;

#[derive(Clone, Debug)]
pub struct EnvironmentColorState {
    key: StateKey,
    value: Color,
    subscribers: SubscriberList<Color>,
}

impl EnvironmentColorState {
    pub fn new(key: EnvironmentColor) -> Self {
        EnvironmentColorState {
            key: StateKey::Color(key),
            value: Color::Rgba(0.0, 0.0, 0.0, 1.0),
            subscribers: SubscriberList::new(),
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

impl crate::state::NewStateSync for EnvironmentColorState {
    fn sync(&mut self, env: &mut Environment) {
        if let Some(color) = env.get_color(&self.key) {
            if self.value != color {
                self.value = color;
                self.subscribers.notify(&color)
            }
        }
    }
}

impl Listenable<Color> for EnvironmentColorState {
    fn subscribe(&self, subscriber: Box<dyn Listener<Color>>) {
        self.subscribers.add_subscriber(subscriber)
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
        self.notify();
    }

    fn notify(&self) {
        self.subscribers.notify(&self.value)
    }
}
