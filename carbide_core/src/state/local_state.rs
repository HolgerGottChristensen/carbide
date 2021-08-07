use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use uuid::Uuid;

use crate::environment::environment::Environment;
use crate::prelude::value_cell::{ValueRef, ValueRefMut};
use crate::state::{State, StateContract, TState};
use crate::state::global_state::{GlobalStateContainer, GlobalStateContract};
use crate::state::state_key::StateKey;
use crate::state::value_cell::ValueCell;
use crate::state::widget_state::WidgetState;

type InnerState<T> = Rc<ValueCell<T>>;

#[derive(Clone)]
pub struct LocalState<T> where T: StateContract {
    key: StateKey,
    value: InnerState<T>,
}

impl<T: StateContract + 'static> LocalState<T> {
    pub fn new(value: T) -> Self {
        LocalState {
            key: StateKey::Uuid(Uuid::new_v4()),
            value: Rc::new(ValueCell::new(value)),
        }
    }
}

impl<T: StateContract + 'static> State<T> for LocalState<T> {
    fn capture_state(&mut self, _: &mut Environment) {}

    fn release_state(&mut self, _: &mut Environment) {}

    fn value(&self) -> ValueRef<T> {
        self.value.borrow()
    }

    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.value.borrow_mut()
    }
}

impl<T: StateContract + 'static> Debug for LocalState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State::LocalState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<T: StateContract + 'static> Into<TState<T>> for Box<LocalState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}