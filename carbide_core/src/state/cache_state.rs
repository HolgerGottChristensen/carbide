use crate::environment::Environment;
use crate::state::{InnerState, NewStateSync, ReadState, RState, State, StateContract, TState, ValueCell, ValueRef, ValueRefMut};

#[derive(Clone)]
pub struct CacheRState<T> where T: StateContract {
    state: RState<T>,
    inner_value: InnerState<Option<T>>,
}

impl<T: StateContract> CacheRState<T> {
    pub fn new(state: RState<T>) -> RState<T> {
        RState::new(Box::new(CacheRState {
            state,
            inner_value: InnerState::new(ValueCell::new(None))
        }))
    }
}

impl<T: StateContract> NewStateSync for CacheRState<T> {
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

impl<T: StateContract> ReadState<T> for CacheRState<T> {
    fn value(&self) -> ValueRef<T> {
        ValueRef::map(self.inner_value.borrow(), |v| {v.as_ref().expect("Tried to get value without having synced first. Maps are not initialized before the first sync")})
    }
}

impl<T: StateContract> core::fmt::Debug for CacheRState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheRState")
            .field("initialized", &self.inner_value.borrow().is_some())
            .field("times_shared", &InnerState::strong_count(&self.inner_value))
            .field("state", &self.state)
            .finish()
    }
}


#[derive(Clone)]
pub struct CacheTState<T> where T: StateContract {
    state: TState<T>,
    inner_value: InnerState<Option<T>>,
}

impl<T: StateContract> CacheTState<T> {
    pub fn new(state: TState<T>) -> TState<T> {
        TState::new(Box::new(CacheTState {
            state,
            inner_value: InnerState::new(ValueCell::new(None))
        }))
    }
}

impl<T: StateContract> NewStateSync for CacheTState<T> {
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

impl<T: StateContract> ReadState<T> for CacheTState<T> {
    fn value(&self) -> ValueRef<T> {
        ValueRef::map(self.inner_value.borrow(), |v| {v.as_ref().expect("Tried to get value without having synced first. Maps are not initialized before the first sync")})
    }
}

impl<T: StateContract> State<T> for CacheTState<T> {
    fn value_mut(&mut self) -> ValueRefMut<T> {
        panic!("You can not set the value of a map state this way. Please use the set_state macro instead")
    }

    /// Set value will only update its containing state if the map_rev is specified.
    #[allow(unused_parens)]
    fn set_value(&mut self, value: T) {
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

    fn update_dependent(&mut self) {}
}

impl<T: StateContract> core::fmt::Debug for CacheTState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheTState")
            .field("initialized", &self.inner_value.borrow().is_some())
            .field("times_shared", &InnerState::strong_count(&self.inner_value))
            .field("state", &self.state)
            .finish()
    }
}