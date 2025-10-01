use crate::environment::{Environment};
use crate::state::{AnyReadState, AnyState, InnerState, StateSync, ReadState, State, StateContract, ValueCell, ValueRef, ValueRefMut};

#[derive(Clone)]
pub struct CachedReadState<T, S> where T: StateContract, S: ReadState<T=T> {
    state: S,
    inner_value: InnerState<Option<T>>,
}

impl<T: StateContract, S: ReadState<T=T>> CachedReadState<T, S> {
    pub fn new(state: S) -> CachedReadState<T, S> {
        CachedReadState {
            state,
            inner_value: InnerState::new(ValueCell::new(None))
        }
    }
}

impl<T: StateContract, S: ReadState<T=T>> StateSync for CachedReadState<T, S> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let updated = self.state.sync(env);

        let borrowed = &mut *self.inner_value.borrow_mut();

        if let Some(inner) = borrowed {
            if updated {
                match self.state.value() {
                    ValueRef::CellBorrow { .. } => {
                        panic!("Dont cache borrows")
                    }
                    ValueRef::Borrow(_) => {
                        panic!("Dont cache borrows")
                    }
                    ValueRef::Owned(o) => {
                        *inner = o;
                    }
                    ValueRef::Locked(_) => {
                        panic!("Dont cache locked")
                    }
                }
            }
        } else {
            match self.state.value() {
                ValueRef::CellBorrow { .. } => {
                    panic!("Dont cache borrows")
                }
                ValueRef::Borrow(_) => {
                    panic!("Dont cache borrows")
                }
                ValueRef::Owned(o) => {
                    *borrowed = Some(o);
                }
                ValueRef::Locked(_) => {
                    panic!("Dont cache locked")
                }
            }
        }

        updated
    }
}

impl<T: StateContract, S: ReadState<T=T>> AnyReadState for CachedReadState<T, S> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<'_, T> {
        ValueRef::map(self.inner_value.borrow(), |v| {v.as_ref().expect("Tried to get value without having synced first. Maps are not initialized before the first sync")})
    }
}

impl<T: StateContract, S: ReadState<T=T>> core::fmt::Debug for CachedReadState<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheRState")
            .field("initialized", &self.inner_value.borrow().is_some())
            .field("times_shared", &InnerState::strong_count(&self.inner_value))
            .field("state", &self.state)
            .finish()
    }
}


#[derive(Clone)]
pub struct CachedState<T, S> where T: StateContract, S: State<T=T> {
    state: S,
    inner_value: InnerState<Option<T>>,
}

impl<T: StateContract, S: State<T=T>> CachedState<T, S> {
    pub fn new(state: S) -> CachedState<T, S> {
        CachedState {
            state,
            inner_value: InnerState::new(ValueCell::new(None))
        }
    }
}

impl<T: StateContract, S: State<T=T>> StateSync for CachedState<T, S> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        let updated = self.state.sync(env);

        let borrowed = &mut *self.inner_value.borrow_mut();

        if let Some(inner) = borrowed {
            if updated {
                match self.state.value() {
                    ValueRef::CellBorrow { .. } => {
                        panic!("Dont cache borrows")
                    }
                    ValueRef::Borrow(_) => {
                        panic!("Dont cache borrows")
                    }
                    ValueRef::Owned(o) => {
                        *inner = o;
                    }
                    ValueRef::Locked(_) => {
                        panic!("Dont cache locked")
                    }
                }
            }
        } else {
            match self.state.value() {
                ValueRef::CellBorrow { .. } => {
                    panic!("Dont cache borrows")
                }
                ValueRef::Borrow(_) => {
                    panic!("Dont cache borrows")
                }
                ValueRef::Owned(o) => {
                    *borrowed = Some(o);
                }
                ValueRef::Locked(_) => {
                    panic!("Dont cache locked")
                }
            }
        }

        updated
    }
}

impl<T: StateContract, S: State<T=T>> AnyReadState for CachedState<T, S> {
    type T = T;
    fn value_dyn(&self) -> ValueRef<'_, T> {
        ValueRef::map(self.inner_value.borrow(), |v| {v.as_ref().expect("Tried to get value without having synced first. Maps are not initialized before the first sync")})
    }
}

impl<T: StateContract, S: State<T=T>> AnyState for CachedState<T, S> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<'_, T> {
        panic!("You can not set the value of a map state this way. Please use the set_state macro instead")
    }

    /// Set value will only update its containing state if the map_rev is specified.
    #[allow(unused_parens)]
    fn set_value_dyn(&mut self, value: T) {
        self.state.set_value(value);

        let borrowed = &mut *self.inner_value.borrow_mut();

        if let Some(inner) = borrowed {
            match self.state.value() {
                ValueRef::CellBorrow { .. } => {
                    panic!("Dont cache borrows")
                }
                ValueRef::Borrow(_) => {
                    panic!("Dont cache borrows")
                }
                ValueRef::Owned(o) => {
                    *inner = o;
                }
                ValueRef::Locked(_) => {
                    panic!("Dont cache locked")
                }
            }
        } else {
            match self.state.value() {
                ValueRef::CellBorrow { .. } => {
                    panic!("Dont cache borrows")
                }
                ValueRef::Borrow(_) => {
                    panic!("Dont cache borrows")
                }
                ValueRef::Owned(o) => {
                    *borrowed = Some(o);
                }
                ValueRef::Locked(_) => {
                    panic!("Dont cache locked")
                }
            }
        }
    }
}

impl<T: StateContract, S: State<T=T>> core::fmt::Debug for CachedState<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheTState")
            .field("initialized", &self.inner_value.borrow().is_some())
            .field("times_shared", &InnerState::strong_count(&self.inner_value))
            .field("state", &self.state)
            .finish()
    }
}