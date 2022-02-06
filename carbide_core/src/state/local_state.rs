use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::rc::Rc;

use uuid::Uuid;
use carbide_core::state::state_sync::NewStateSync;
use carbide_core::state::subscriber::Listenable;

use crate::environment::Environment;
use crate::state::{InnerState, ReadState, State, StateContract, Listener, SubscriberList, TState};
use crate::state::state_key::StateKey;
use crate::state::value_cell::{ValueCell, ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

/// # Local state
/// The local state is used as a shared state between multiple widgets within the same widget tree.
/// When cloning this the inner state will be shared between the original and the clone.
/// The same is the case for the list of listeners.
#[derive(Clone)]
pub struct LocalState<T> where T: StateContract {
    /// A list of listeners that contains all the states that should
    /// change whenever this state changes.
    listeners: SubscriberList<T>,
    /// The shared state
    inner_value: InnerState<T>,
}

impl<T: StateContract> LocalState<T> {
    /// Returns a new local state containing the value provided.
    /// Returns the local state wrapped within a WidgetState.
    pub fn new(value: T) -> TState<T> {
        WidgetState::new(Self::new_raw(value))
    }

    /// Returns a new local state containing the value provided.
    /// Often you should use `new` when creating states, but this can be used to get the state
    /// within a box.
    pub fn new_raw(value: T) -> Box<Self> {
        Box::new(LocalState {
            listeners: SubscriberList::new(),
            inner_value: Rc::new(ValueCell::new(value)),
        })
    }
}

/// Local state does not need to do any updating when sync is called. Also it does not depend
/// on any other states and therefore the event can be ignored.
impl<T: StateContract> NewStateSync for LocalState<T> {}

/// Local state is listenable. You are able to subscribe for notifications whenever this state
/// changes.
impl<T: StateContract> Listenable<T> for LocalState<T> {
    fn subscribe(&self, listener: Box<dyn Listener<T>>) {
        self.listeners.add_subscriber(listener)
    }
}

impl<T: StateContract> ReadState<T> for LocalState<T> {
    fn value(&self) -> ValueRef<T> {
        self.inner_value.borrow()
    }
}

impl<T: StateContract> State<T> for LocalState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        self.inner_value.borrow_mut()
    }

    fn set_value(&mut self, value: T) {
        *self.inner_value.borrow_mut() = value;
        self.notify();
    }

    fn notify(&self) {
        self.listeners.notify(&*self.inner_value.borrow());
    }
}

impl<T: StateContract> Debug for LocalState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalState")
            .field("value", &*self.value())
            .finish()
    }
}
