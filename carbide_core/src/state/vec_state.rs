use std::borrow::BorrowMut;
use std::fmt::{Debug, Formatter};
use carbide_core::prelude::{NewStateSync, Listenable, Listener, Id};

use crate::prelude::Environment;
use crate::state::{ReadState, StateContract, SubscriberList, TState, UsizeState};
use crate::state::state::State;
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

/// # Vector state
/// Vector state is a state mapping from a state of `Vec<T>` and a state of `usize` to a state of `T`.
/// Note: TODO
///
/// This state is ['Listenable'] and handles the subscriptions such that a change in either the
/// `usize` state or the `Vec<T>` state changes, the listener will receive a notification.
#[derive(Clone)]
pub struct VecState<T>
    where
        T: StateContract,
{
    /// The state that is evaluated whenever trying to get the index within the vec.
    index_state: UsizeState,
    index_state_listener_id: Id,
    /// The state containing the vec.
    vec_state: TState<Vec<T>>,
    vec_state_listener_id: Id,
    /// The list of subscribers to notify whenever the state changes.
    listeners: SubscriberList<T>,

}

impl<T: StateContract> VecState<T> {
    pub fn new(vec: impl Into<TState<Vec<T>>>, index: impl Into<UsizeState>) -> VecState<T> {
        let list = SubscriberList::new();
        let list_for_listener1 = list.clone();
        let list_for_listener2 = list.clone();
        let list = SubscriberList::new();
        let vec_state = vec.into();
        let vec_state_for_listener = vec_state.clone();
        let usize_state = index.into();
        let usize_state_for_listener = usize_state.clone();

        let vec_state_listener_id = vec_state.subscribe(Box::new(move |val: &Vec<T>| {
            let index = *usize_state_for_listener.value();
            // When the parent changes we should notify the listeners to this state.
            list_for_listener1.notify(&val[index])
        }));

        let usize_state_listener_id = usize_state.subscribe(Box::new(move |index: &usize| {
            let vec = vec_state_for_listener.value();
            // When the parent changes we should notify the listeners to this state.
            list_for_listener2.notify(&vec[*index])
        }));

        VecState {
            index_state: usize_state,
            index_state_listener_id: usize_state_listener_id,
            vec_state: vec_state.into(),
            vec_state_listener_id,
            listeners: list,
        }
    }
}

impl<T: StateContract> Drop for VecState<T> {
    fn drop(&mut self) {
        // Make sure to unsubscribe
        self.vec_state.unsubscribe(&self.vec_state_listener_id);
        self.index_state.unsubscribe(&self.index_state_listener_id);
    }
}

impl<T: StateContract> NewStateSync for VecState<T> {
    fn sync(&mut self, env: &mut Environment) {
        self.index_state.sync(env);
        self.vec_state.sync(env);
    }
}

impl<T: StateContract> Listenable<T> for VecState<T> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) -> Id {
        self.listeners.add_subscriber(subscriber)
    }

    fn unsubscribe(&self, id: &Id) {
        self.listeners.remove_subscriber(id)
    }
}

impl<T: StateContract> ReadState<T> for VecState<T> {
    fn value(&self) -> ValueRef<T> {
        let index = *self.index_state.value();
        ValueRef::map(self.vec_state.value(), |a| { &a[index] })
    }
}

impl<T: StateContract> State<T> for VecState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        let index = *self.index_state.value();
        ValueRefMut::map(self.vec_state.value_mut(), |a| &mut a[index])
    }

    fn set_value(&mut self, value: T) {
        *self.value_mut() = value;
        self.vec_state.notify(); // This will also notify self's listeners because it is linked.
    }

    fn notify(&self) {
        self.listeners.notify(&*self.value());
    }
}

impl<T: StateContract> Debug for VecState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecState")
            .field("value", &*self.value())
            .field("index", &*self.index_state.value())
            .finish()
    }
}

impl<T: StateContract> Into<TState<T>> for VecState<T> {
    fn into(self) -> TState<T> {
        WidgetState::new(Box::new(self))
    }
}

impl<T: StateContract> Into<TState<T>> for Box<VecState<T>> {
    fn into(self) -> TState<T> {
        WidgetState::new(self)
    }
}
