use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;
use std::rc::{Rc, Weak};
use crate::state::{InnerState, Listener, StateContract, ValueCell};

pub trait Listenable<T: StateContract> {
    fn subscribe(&self, subscriber: Box<dyn Listener<T>>);
}


#[derive(Clone)]
pub struct SubscriberList<T: StateContract>(InnerState<Vec<Box<dyn Listener<T>>>>);

impl<T: StateContract> SubscriberList<T> {
    pub fn new() -> SubscriberList<T> {
        Self(Rc::new(ValueCell::new(vec![])))
    }

    pub fn add_subscriber(&self, subscriber: Box<dyn Listener<T>>) {
        self.0.borrow_mut().push(subscriber);
    }

    pub fn len(&self) -> usize {
        // Check if we should keep the listener in the list.
        self.0.borrow_mut().retain(|a| a.keep());

        self.0.borrow().len()
    }

    /// Notify all the listeners in the list
    pub fn notify(&self, value: &T) {
        // Check if we should keep the listener in the list.
        self.0.borrow_mut().retain(|a| a.keep());

        // Call change for all listeners.
        // TODO the operations here and above in notify can be replaced when retain_mut gets added
        // to the standard lib. https://github.com/rust-lang/rust/issues/90829
        self.0.borrow_mut()
            .iter_mut()
            .for_each(|a| a.change(value));
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
pub struct WeakSubscriberList<T: StateContract>(Weak<ValueCell<Vec<Box<dyn Listener<T>>>>>);

impl<T: StateContract> WeakSubscriberList<T> {
    /// Notify all the listeners in the list if the list is still available
    pub fn try_notify(&self, value: &T) {
        if let Some(inner) = self.0.upgrade() {
            // Check if we should keep the listener in the list.
            inner.borrow_mut().retain(|a| a.keep());

            // Call change for all listeners.
            // TODO the operations here and above in notify can be replaced when retain_mut gets added
            // to the standard lib. https://github.com/rust-lang/rust/issues/90829
            inner.borrow_mut()
                .iter_mut()
                .for_each(|a| a.change(value));
        }
    }
}

impl<T: StateContract> Debug for WeakSubscriberList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

