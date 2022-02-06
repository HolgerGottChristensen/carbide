use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use dyn_clone::DynClone;
use carbide_core::prelude::{NewStateSync, Listenable};
use carbide_core::state::Listener;

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::{InnerState, LocalState, MapListener, ReadState, RState, State, StringState, SubscriberList, ValueCell, ValueState};
use crate::state::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct NewMapState<FROM, TO>
    where
        FROM: StateContract,
        TO: StateContract,
{
    state: TState<FROM>,
    get: fn(&FROM) -> TO,
    replace: fn(TO, &FROM) -> FROM,
    inner_value: InnerState<TO>,
    listeners: SubscriberList<TO>,
}

impl<FROM: StateContract, TO: StateContract> NewMapState<FROM, TO> {
    pub fn new<M1: Into<TState<FROM>>>(state: M1, map: fn(&FROM) -> TO, rev_map: fn(TO, &FROM) -> FROM) -> TState<TO> {
        let state = state.into();
        let value = (map)(&*state.value());

        let inner = InnerState::new(ValueCell::new(value));
        let list = SubscriberList::new();

        let res = NewMapState {
            state: state.clone(),
            get: map,
            replace: rev_map,
            inner_value: inner.clone(),
            listeners: list.clone()
        };

        let map_listener = MapListener::<FROM, TO, fn(&FROM) -> TO>::new(inner, map, list);

        state.subscribe(Box::new(map_listener));

        res.into()
    }
}

impl<FROM: StateContract, TO: StateContract> NewStateSync for NewMapState<FROM, TO> {}

impl<FROM: StateContract, TO: StateContract> Listenable<TO> for NewMapState<FROM, TO> {
    fn subscribe(&self, subscriber: Box<dyn Listener<TO>>) {
        self.listeners.add_subscriber(subscriber);
    }
}

impl<FROM: StateContract, TO: StateContract> ReadState<TO> for NewMapState<FROM, TO> {
    fn value(&self) -> ValueRef<TO> {
        self.inner_value.borrow()
    }
}

impl<FROM: StateContract, TO: StateContract> State<TO> for NewMapState<FROM, TO> {
    fn value_mut(&mut self) -> ValueRefMut<TO> {
        panic!("This method should no longer be used")
    }

    /// Set value will only update its containing state if the map_rev is specified.
    fn set_value(&mut self, value: TO) {
        let from = (self.replace)(value, &*self.state.value());
        self.state.set_value(from);
    }

    fn notify(&self) {
        self.listeners.notify(&*self.value())
    }

    fn update_dependent(&mut self) {}
}

impl<FROM: StateContract, TO: StateContract> Debug for NewMapState<FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract, TO: StateContract> Into<TState<TO>>
for NewMapState<FROM, TO>
{
    fn into(self) -> TState<TO> {
        WidgetState::new(Box::new(self))
    }
}
