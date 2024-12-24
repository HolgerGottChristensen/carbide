use crate::environment::{EnvironmentStack, Key, Keyable};
use carbide::state::{AnyReadState, StateSync, ValueRef};

#[derive(Debug)]
pub struct KeyState<K: Key> where K::Value: Clone {
    current: K::Value,
    default: K::Value,
}

impl<K: Key> KeyState<K> where K::Value: Clone {
    pub fn new(default: K::Value) -> KeyState<K> {
        KeyState {
            current: default.clone(),
            default,
        }
    }
}


impl<K: Key> StateSync for KeyState<K> where K::Value: Clone {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        self.current = env.get::<K>().cloned().unwrap_or(self.default.clone());
        true
    }
}

impl<K: Key> AnyReadState for KeyState<K> where K::Value: Clone {
    type T = K::Value;
    fn value_dyn(&self) -> ValueRef<K::Value> {
        ValueRef::Owned(self.current.clone())
    }
}

impl<K: Key> Clone for KeyState<K> where K::Value: Clone {
    fn clone(&self) -> Self {
        KeyState {
            current: self.current.clone(),
            default: self.default.clone(),
        }
    }
}