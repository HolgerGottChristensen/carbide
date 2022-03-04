use std::marker::PhantomData;
use std::rc::{Rc, Weak};
use crate::prelude::StateContract;
use crate::state::listener::listener::Listener;
use crate::state::util::subscriber::WeakSubscriberList;
use crate::state::{InnerState, RState, SubscriberList, ValueCell};
use crate::state::readonly::ReadMap;

#[derive(Clone)]
pub struct IndexListener<T> where T: StateContract {
    weak: Weak<ValueCell<Vec<T>>>,
    index: RState<usize>,
    listeners: WeakSubscriberList<T>,
}
/*
impl<T: StateContract> IndexListener<T> {
    pub fn new(state: InnerState<Vec<T>>, index: impl Into<RState<usize>>, list: SubscriberList<T>) -> Self {
        IndexListener {
            weak: Rc::downgrade(&state),
            index: index.into(),
            listeners: list.downgrade(),
        }
    }
}

impl<T: StateContract> Listener<Vec<T>> for IndexListener<T> {
    fn change(&mut self, value: &Vec<T>) {
        if let Some(inner) = self.weak.upgrade() {
            self.listeners.try_notify(&*inner.borrow());
        }
    }

    fn keep(&self) -> bool {
        self.weak.strong_count() != 0
    }
}*/