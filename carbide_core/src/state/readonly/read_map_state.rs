use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use dyn_clone::DynClone;
use carbide_core::prelude::{NewStateSync, Listenable};
use carbide_core::state::Listener;

use crate::environment::Environment;
use crate::prelude::{StateContract, TState};
use crate::state::{InnerState, LocalState, MapListener, ReadState, RState, State, StateExt, StringState, SubscriberList, ValueCell, ValueState};
use crate::state::readonly::{ReadMap, ReadWidgetState};
use crate::state::util::value_cell::{ValueRef, ValueRefMut};
use crate::state::widget_state::WidgetState;

#[derive(Clone)]
pub struct ReadMapState<FROM, TO>
    where
        FROM: StateContract,
        TO: StateContract
{
    state: RState<FROM>,
    value: InnerState<TO>,
    subscribers: SubscriberList<TO>
}

impl<FROM: StateContract, TO: StateContract> ReadMapState<FROM, TO> {
    pub fn new<M1: Into<RState<FROM>>, MAP: ReadMap<FROM, TO>>(state: M1, map: MAP) -> RState<TO> {
        let state = state.into();
        let value = map.map(&*state.value());

        let inner_state = InnerState::new(ValueCell::new(value));
        let list = SubscriberList::new();

        let res = ReadMapState {
            state: state.clone(),
            value: inner_state.clone(),
            subscribers: list.clone(),
        };

        let listener = MapListener::new(inner_state, map, list);

        state.subscribe(Box::new(listener));

        res.into()
    }
}

impl<FROM: StateContract, TO: StateContract> NewStateSync for ReadMapState<FROM, TO> {
    fn sync(&mut self, env: &mut Environment) {
        self.state.sync(env)
    }
}

impl<FROM: StateContract, TO: StateContract> Listenable<TO> for ReadMapState<FROM, TO> {
    fn subscribe(&self, subscriber: Box<dyn Listener<TO>>) {
        self.subscribers.add_subscriber(subscriber)
    }
}

impl<FROM: StateContract, TO: StateContract> ReadState<TO> for ReadMapState<FROM, TO> {
    fn value(&self) -> ValueRef<TO> {
        self.value.borrow()
    }
}

impl<FROM: StateContract, TO: StateContract> Debug for ReadMapState<FROM, TO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapState")
            .field("value", &*self.value())
            .finish()
    }
}

impl<FROM: StateContract, TO: StateContract> Into<RState<TO>> for ReadMapState<FROM, TO> {
    fn into(self) -> RState<TO> {
        ReadWidgetState::new(Box::new(self))
    }
}
