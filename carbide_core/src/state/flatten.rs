use crate::environment::{Environment};
use crate::state::{AnyReadState, StateSync, ReadState, StateContract, ValueRef};

#[derive(Debug, Clone)]
pub struct FlattenedReadState<T, S, SS>
where
    T: StateContract,
    S: ReadState<T=T>,
    SS: ReadState<T=S>,
{
    state: SS,
    current: Option<S>,
}

/*impl<T: StateContract> FlattenedReadState<T> {
    pub fn new(s: impl Into<TState<TState<T>>>) -> TState<T> {
        TState::new(Box::new(Flatten {
            state: s.into(),
            current_inner: None,
        }))
    }
}*/

impl<T: StateContract, S: ReadState<T=T>, SS: ReadState<T=S>> StateSync for FlattenedReadState<T, S, SS> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        if self.state.sync(env) {
            self.current = Some(self.state.value().clone());
            self.current.as_mut().unwrap().sync(env)
        } else {
            false
        }
    }
}

impl<T: StateContract, S: ReadState<T=T>, SS: ReadState<T=S>> AnyReadState for FlattenedReadState<T, S, SS> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        self.current.as_ref().unwrap().value()
    }
}
