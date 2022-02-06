use dyn_clone::DynClone;
use crate::state::StateContract;

/// A listener that will be called when the value has changed.
pub trait Listener<T>: DynClone where T: StateContract {

    /// This method is called when the value listening to has changed.
    fn change(&mut self, value: &T);

    /// Should return true if the listener should be retained. If false is returned the listener
    /// will no longer be called and removed from any subscriber lists.
    fn keep(&self) -> bool;
}

impl<K, T: StateContract> Listener<T> for K where K: Fn(&T) + Clone {
    fn change(&mut self, value: &T) { self(value) }

    fn keep(&self) -> bool {
        true
    }
}