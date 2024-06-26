use carbide_core::state::AnyState;
use crate::environment::Environment;
use crate::state::{AnyReadState, NewStateSync, ReadState, State, StateContract, TState, ValueRef, ValueRefMut};

#[derive(Debug, Clone)]
pub struct Flatten<T>
where
    T: StateContract,
{
    state: TState<TState<T>>,
    current_inner: Option<TState<T>>,
}

impl<T: StateContract> Flatten<T> {
    pub fn new(s: impl Into<TState<TState<T>>>) -> TState<T> {
        TState::new(Box::new(Flatten {
            state: s.into(),
            current_inner: None,
        }))
    }
}

impl<T: StateContract> NewStateSync for Flatten<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        if self.state.sync(env) {
            self.current_inner = Some(self.state.value().clone());
            self.current_inner.as_mut().unwrap().sync(env)
        } else {
            false
        }
    }
}

impl<T: StateContract> AnyReadState for Flatten<T> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        self.current_inner
            .as_ref()
            .expect("Tried to get value without having synced first.")
            .value()
    }
}

impl<T: StateContract> AnyState for Flatten<T> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<T> {
        panic!("You can not set the value of a map state this way. Please use the set_state macro instead")
    }

    fn set_value_dyn(&mut self, value: T) {
        self.current_inner
            .as_mut()
            .expect("Tried to get value without having synced first.")
            .set_value(value)
    }
}
