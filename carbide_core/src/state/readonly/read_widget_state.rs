use std::collections::HashSet;
use std::fmt;
use std::fmt::{Debug, Formatter};

use dyn_clone::DynClone;
use carbide_core::prelude::{NewStateSync, Listenable, Id};

use crate::prelude::{Environment, WidgetState};
use crate::state::{MapState, ReadState, StateContract, StateExt, Listener, TState, UsizeState};
use crate::state::readonly::ignore_write_state::IgnoreWritesState;
pub use crate::state::State;
use crate::state::util::value_cell::{ValueRef, ValueRefMut};

pub enum ReadWidgetState<T> {
    ReadState(Box<dyn ReadState<T>>),
    ReadWriteState(TState<T>)
}

impl<T: StateContract> ReadWidgetState<T> {
    pub fn new(item: Box<dyn ReadState<T>>) -> ReadWidgetState<T> {
        ReadWidgetState::ReadState(item)
    }

    pub fn new_from_read_write_state(item: TState<T>) -> ReadWidgetState<T> {
        ReadWidgetState::ReadWriteState(item)
    }

    pub fn ignore_writes(&self) -> TState<T> {
        IgnoreWritesState::new(self.clone())
    }
}

impl<T: StateContract> Debug for ReadWidgetState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ReadWidgetState::ReadState(n) => {
                n.fmt(f)
            }
            ReadWidgetState::ReadWriteState(n) => {
                n.fmt(f)
            }
        }

    }
}

impl<T: StateContract> Clone for ReadWidgetState<T> {
    fn clone(&self) -> Self {
        match self {
            ReadWidgetState::ReadState(n) => {
                ReadWidgetState::ReadState(n.clone())
            }
            ReadWidgetState::ReadWriteState(n) => {
                ReadWidgetState::ReadWriteState(n.clone())
            }
        }
    }
}

impl<T: StateContract> Into<ReadWidgetState<T>> for Box<dyn ReadState<T>> {
    fn into(self) -> ReadWidgetState<T> {
        ReadWidgetState::ReadState(self)
    }
}

impl<T: StateContract> NewStateSync for ReadWidgetState<T> {
    fn sync(&mut self, env: &mut Environment) {
        match self {
            ReadWidgetState::ReadState(r) => {
                r.sync(env)
            }
            ReadWidgetState::ReadWriteState(rw) => {
                rw.sync(env)
            }
        }
    }
}

impl<T: StateContract> Listenable<T> for ReadWidgetState<T> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) -> Id {
        match self {
            ReadWidgetState::ReadState(n) => {
                n.subscribe(subscriber)
            }
            ReadWidgetState::ReadWriteState(n) => {
                n.subscribe(subscriber)
            }
        }
    }

    fn unsubscribe(&self, id: &Id) {
        match self {
            ReadWidgetState::ReadState(n) => {
                n.unsubscribe(id)
            }
            ReadWidgetState::ReadWriteState(n) => {
                n.unsubscribe(id)
            }
        }
    }
}

impl<T: StateContract> ReadState<T> for ReadWidgetState<T> {
    fn value(&self) -> ValueRef<T> {
        match self {
            ReadWidgetState::ReadState(n) => {
                n.value()
            }
            ReadWidgetState::ReadWriteState(n) => {
                n.value()
            }
        }
    }

    /*fn value_changed(&mut self) {
        match self {
            ReadWidgetState::ReadState(n) => {
                n.value_changed()
            }
            ReadWidgetState::ReadWriteState(n) => {
                n.value_changed()
            }
        }
    }*/
}