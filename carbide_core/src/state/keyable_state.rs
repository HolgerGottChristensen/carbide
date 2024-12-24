use crate::environment::{EnvironmentStack, Keyable};
use crate::state::ReadState;
use crate::state::{AnyReadState, StateSync, ValueRef};

#[derive(Debug)]
pub struct KeyableState<K: Keyable, S: ReadState<T=K>> where K::Output: Clone {
    state: S,
    current: K::Output,
    default: K::Output,
}

impl<K: Keyable, S: ReadState<T=K>> KeyableState<K, S> where K::Output: Clone {
    pub fn new(state: S, default: K::Output) -> KeyableState<K, S> {
        KeyableState {
            state,
            current: default.clone(),
            default,
        }
    }
}


impl<K: Keyable, S: ReadState<T=K>> StateSync for KeyableState<K, S>
where K::Output: Clone {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        self.state.sync(env);
        self.current = env.value(&*self.state.value()).unwrap_or(self.default.clone());
        true
    }
}

impl<K: Keyable, S: ReadState<T=K>> AnyReadState for KeyableState<K, S>
where K::Output: Clone {
    type T = K::Output;
    fn value_dyn(&self) -> ValueRef<K::Output> {
        ValueRef::Owned(self.current.clone())
    }
}

impl<K: Keyable, S: ReadState<T=K>> Clone for KeyableState<K, S>
where K::Output: Clone {
    fn clone(&self) -> Self {
        KeyableState {
            state: self.state.clone(),
            current: self.current.clone(),
            default: self.default.clone(),
        }
    }
}