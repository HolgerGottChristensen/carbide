use std::fmt;
use std::fmt::{Debug, Formatter};

use crate::environment::Environment;
use crate::state::{AnyReadState, NewStateSync, RState, StateContract, TState, ValueRef, ValueState, WidgetState};

pub enum ReadWidgetState<T>
where
    T: StateContract,
{
    ReadState(Box<dyn AnyReadState<T=T>>),
    ReadWriteState(TState<T>),
}

impl<T: StateContract> ReadWidgetState<T> {
    pub fn new(item: Box<dyn AnyReadState<T=T>>) -> ReadWidgetState<T> {
        ReadWidgetState::ReadState(item)
    }

    pub fn new_from_read_write_state(item: TState<T>) -> ReadWidgetState<T> {
        ReadWidgetState::ReadWriteState(item)
    }

    /*pub fn ignore_writes(&self) -> TState<T> {
        IgnoreWritesState::new(self.clone())
    }*/
}

impl<T: StateContract> Debug for ReadWidgetState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReadWidgetState::ReadState(n) => n.fmt(f),
            ReadWidgetState::ReadWriteState(n) => n.fmt(f),
        }
    }
}

impl<T: StateContract> Clone for ReadWidgetState<T> {
    fn clone(&self) -> Self {
        match self {
            ReadWidgetState::ReadState(n) => ReadWidgetState::ReadState(n.clone()),
            ReadWidgetState::ReadWriteState(n) => ReadWidgetState::ReadWriteState(n.clone()),
        }
    }
}

impl<T: StateContract> Into<ReadWidgetState<T>> for Box<dyn AnyReadState<T=T>> {
    fn into(self) -> ReadWidgetState<T> {
        ReadWidgetState::ReadState(self)
    }
}

impl<T: StateContract> NewStateSync for ReadWidgetState<T> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        match self {
            ReadWidgetState::ReadState(r) => r.sync(env),
            ReadWidgetState::ReadWriteState(rw) => rw.sync(env),
        }
    }
}

impl<T: StateContract> AnyReadState for ReadWidgetState<T> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<T> {
        match self {
            ReadWidgetState::ReadState(n) => n.value_dyn(),
            ReadWidgetState::ReadWriteState(n) => n.value_dyn(),
        }
    }
}

impl<T: StateContract> From<T> for RState<T> {
    fn from(t: T) -> Self {
        ReadWidgetState::ReadWriteState(WidgetState::Value(ValueState::new(t)))
    }
}

/*impl From<u32> for RState<f64> {
    fn from(t: u32) -> Self {
        ValueState::new(t as f64).into()
    }
}*/

