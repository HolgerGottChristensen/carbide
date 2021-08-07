use std::cell::{Ref, RefCell, RefMut};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::prelude::Environment;
use crate::prelude::value_cell::ValueRef;
use crate::state::{State, StateContract, TState};
use crate::state::value_cell::{ValueCell, ValueRefMut};
use crate::state::widget_state::WidgetState;

// The global state needs to implement clone because the widgets do, and for them to be clone
// All the generic types need to implement it as well. The global state should never in practise
// be cloned, because that would most likely be very expensive.
pub trait GlobalStateContract: 'static + Clone + std::fmt::Debug {}

impl<T> GlobalStateContract for T where T: 'static + Clone + std::fmt::Debug {}

pub type GlobalStateContainer<GS: GlobalStateContract> = Rc<RefCell<GS>>;

type InnerState<T> = Rc<ValueCell<T>>;

#[derive(Clone)]
pub struct GState<T> where T: StateContract {
    value: InnerState<T>,
}

impl<T: StateContract> GState<T> {
    pub fn new(env: &Environment) -> Self {
        GState {
            value: env.get_global_state::<T>()
        }
    }
}

impl<'a, T: StateContract> State<T> for GState<T> {
    fn capture_state(&mut self, _: &mut Environment) {}

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<T> {
        self.value.borrow()
    }

    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.value.borrow_mut()
    }
}

impl<T: StateContract> Debug for GState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::GlobalState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<T: StateContract + 'static> Into<TState<T>> for Box<GState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}