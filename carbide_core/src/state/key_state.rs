use crate::environment::{Environment, EnvironmentKey};
use crate::state::{AnyReadState, StateSync, ValueRef};

#[derive(Debug)]
pub struct KeyState<K: EnvironmentKey> where K::Value: Clone {
    current: K::Value,
    default: K::Value,
}

impl<K: EnvironmentKey> KeyState<K> where K::Value: Clone {
    pub fn new(default: K::Value) -> KeyState<K> {
        KeyState {
            current: default.clone(),
            default,
        }
    }
}


impl<K: EnvironmentKey> StateSync for KeyState<K> where K::Value: Clone {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.current = env.get::<K>().cloned().unwrap_or(self.default.clone());
        true
    }
}

impl<K: EnvironmentKey> AnyReadState for KeyState<K> where K::Value: Clone {
    type T = K::Value;
    fn value_dyn(&self) -> ValueRef<K::Value> {
        ValueRef::Owned(self.current.clone())
    }
}

impl<K: EnvironmentKey> Clone for KeyState<K> where K::Value: Clone {
    fn clone(&self) -> Self {
        KeyState {
            current: self.current.clone(),
            default: self.default.clone(),
        }
    }
}