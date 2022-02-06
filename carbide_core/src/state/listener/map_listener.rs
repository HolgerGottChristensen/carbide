use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use crate::prelude::StateContract;
use crate::state::listener::listener::Listener;
use crate::state::subscriber::WeakSubscriberList;
use crate::state::{InnerState, SubscriberList, ValueCell};
use crate::state::readonly::ReadMap;

#[derive(Clone)]
pub struct MapListener<FROM, TO, MAP> where FROM: StateContract, TO: StateContract, MAP: ReadMap<FROM, TO> {
    weak: Weak<ValueCell<TO>>,
    get: MAP,
    listeners: WeakSubscriberList<TO>,
    phantom: PhantomData<FROM>,
}

impl<FROM: StateContract, TO: StateContract, MAP: ReadMap<FROM, TO>> MapListener<FROM, TO, MAP> {
    pub fn new(state: InnerState<TO>, get: MAP, list: SubscriberList<TO>) -> Self {
        MapListener {
            weak: Rc::downgrade(&state),
            get,
            listeners: list.downgrade(),
            phantom: Default::default()
        }
    }
}


impl<FROM: StateContract, TO: StateContract, MAP: ReadMap<FROM, TO>> Listener<FROM> for MapListener<FROM, TO, MAP> {
    fn change(&mut self, value: &FROM) {
        if let Some(inner) = self.weak.upgrade() {
            *inner.borrow_mut() = self.get.map(value);
            self.listeners.try_notify(&*inner.borrow());
        }
    }

    fn keep(&self) -> bool {
        self.weak.strong_count() != 0
    }
}