use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use crate::state::{InnerState, Listener, StateContract, ValueCell};
use crate::widget::Id;

pub trait Listenable<T: StateContract> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>) -> Id;

    fn unsubscribe(&self, id: &Id);
}


#[derive(Clone)]
pub struct SubscriberList<T: StateContract>(InnerState<Vec<(Id, Box<dyn Listener<T>>)>>);

impl<T: StateContract> SubscriberList<T> {
    pub fn new() -> SubscriberList<T> {
        Self(Rc::new(ValueCell::new(vec![])))
    }

    pub fn add_subscriber(&self, subscriber: Box<dyn Listener<T>>) -> Id {
        let id = Id::new_v4();
        self.0.borrow_mut().push((id, subscriber));
        id
    }

    pub fn retain(&self) {
        self.0.borrow_mut().retain(|(_, a)| a.keep())
    }

    /// Remove the element within the list in O(n) time. This is not the best we can do, but we
    /// will more likely iterate over this many more times than removing, so that was the performance
    /// trade of decided upon.
    pub fn remove_subscriber(&self, id: &Id) {
        self.0.borrow_mut().retain(|(a, _)| a != id)
    }

    pub fn len(&self) -> usize {
        // Check if we should keep the listener in the list.
        self.retain();

        self.0.borrow().len()
    }

    /// Notify all the listeners in the list
    pub fn notify(&self, value: &T) {
        // Check if we should keep the listener in the list.
        self.0.borrow_mut().retain(|(_, a)| a.keep());

        // Call change for all listeners.
        // TODO the operations here and above in notify can be replaced when retain_mut gets added
        // to the standard lib. https://github.com/rust-lang/rust/issues/90829
        self.0.borrow_mut()
            .iter_mut()
            .for_each(|(_, a)| a.change(value));
    }

    pub(crate) fn downgrade(&self) -> WeakSubscriberList<T> {
        WeakSubscriberList(Rc::downgrade(&self.0))
    }
}

impl<T: StateContract> Debug for SubscriberList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Clone)]
pub struct WeakSubscriberList<T: StateContract>(Weak<ValueCell<Vec<(Id, Box<dyn Listener<T>>)>>>);

impl<T: StateContract> WeakSubscriberList<T> {
    /// Notify all the listeners in the list if the list is still available
    pub fn try_notify(&self, value: &T) {
        if let Some(inner) = self.0.upgrade() {
            // Check if we should keep the listener in the list.
            inner.borrow_mut().retain(|(_, a)| a.keep());

            // Call change for all listeners.
            // TODO the operations here and above in notify can be replaced when retain_mut gets added
            // to the standard lib. https://github.com/rust-lang/rust/issues/90829
            inner.borrow_mut()
                .iter_mut()
                .for_each(|(_, a)| a.change(value));
        }
    }
}

impl<T: StateContract> Debug for WeakSubscriberList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

