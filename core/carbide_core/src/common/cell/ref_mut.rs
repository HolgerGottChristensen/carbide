use std::fmt;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use parking_lot::MappedRwLockWriteGuard;
use carbide::common::cell::value_cell::BorrowRefMut;
use crate::common::cell::r#ref::ValueRef;

pub type RefMut<'a, T> = ValueRefMut<'a, T>;

pub enum ValueRefMut<'a, T: 'a + Debug + Clone + 'static> {
    CellBorrow {
        value: Option<&'a mut T>,
        borrow: Option<BorrowRefMut<'a>>,
    },
    Borrow(Option<&'a mut T>),
    Locked(Option<MappedRwLockWriteGuard<'a, T>>),
    TupleState(Option<Box<dyn FnOnce(T)>>, Option<T>),
    Read(ValueRef<'a, T>),
}

impl<'b, T: Clone + Debug + 'static> ValueRefMut<'b, T> {
    pub fn map<U: Clone + Debug, F>(mut orig: ValueRefMut<'b, T>, f: F) -> ValueRefMut<'b, U>
    where
        F: (FnOnce(&mut T) -> &mut U) + Clone + 'static,
    {
        match orig {
            ValueRefMut::CellBorrow { ref mut value, ref mut borrow } => {
                ValueRefMut::CellBorrow {
                    value: value.take().map(f),
                    borrow: borrow.take(),
                }
            },
            ValueRefMut::Borrow(ref mut value) => {
                ValueRefMut::Borrow(value.take().map(f))
            },
            ValueRefMut::Locked(ref mut value) => {
                ValueRefMut::Locked(Some(MappedRwLockWriteGuard::map(value.take().unwrap(), f)))
            }
            ValueRefMut::TupleState(ref mut state, ref mut value) => {
                let setter = state.take().unwrap();
                let mut value = value.take().unwrap();

                let new = (f.clone())(&mut value).clone();

                let new_setter = move |new: U| {
                    let mut value = value;

                    *f(&mut value) = new;

                    setter(value);
                };

                ValueRefMut::TupleState(Some(Box::new(new_setter)), Some(new))
            }
            ValueRefMut::Read(_) => {
                todo!()
            }
        }
    }

    pub fn apply<U: ?Sized>(mut self, s: &mut U, f: fn(&'b mut T, &mut U)) {
        match self {
            ValueRefMut::CellBorrow { ref mut value, .. } => {
                f(value.take().unwrap(), s)
            }
            ValueRefMut::Locked(ref mut _value) => {
                //MappedRwLockReadGuard::map(value, |a| {f(a, s); a});
                todo!()
            }
            ValueRefMut::Borrow(ref mut value) => {
                f(value.take().unwrap(), s)
            }

            ValueRefMut::TupleState(_, _) => {
                todo!()
            }
            ValueRefMut::Read(_) => {
                todo!()
            }
        }
    }
}

impl<T: Debug + Clone + 'static> Drop for ValueRefMut<'_, T> {
    fn drop(&mut self) {
        //println!("Dropped: {:?}", self);
        match self {
            ValueRefMut::CellBorrow { .. } => {}
            ValueRefMut::Borrow(_) => {}
            ValueRefMut::Locked(_) => {}
            ValueRefMut::TupleState(state, val) => {
                if let Some(state) = state.take() {
                    if let Some(val) = val.take() {
                        state(val);
                    } else {
                        unreachable!("The val should always be Some when the state is Some")
                    }
                }
            }
            ValueRefMut::Read(_) => {}
        }
    }
}

impl<T: Debug + Clone + 'static> Deref for ValueRefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            ValueRefMut::CellBorrow { value, .. } => value.as_ref().unwrap(),
            ValueRefMut::Borrow(value) => value.as_ref().unwrap(),
            ValueRefMut::Locked(value) => value.as_ref().unwrap().deref(),
            ValueRefMut::TupleState(_, val) => val.as_ref().unwrap(),
            ValueRefMut::Read(val) => val
        }
    }
}

impl<T: Debug + Clone + 'static> DerefMut for ValueRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        match self {
            ValueRefMut::CellBorrow { value, .. } => value.as_mut().unwrap(),
            ValueRefMut::Borrow(value) => value.as_mut().unwrap(),
            ValueRefMut::Locked(value) => value.as_mut().unwrap().deref_mut(),
            ValueRefMut::TupleState(_, val) => val.as_mut().unwrap(),
            ValueRefMut::Read(_) => {
                panic!("Ignore writes was used. Cannot deref mut")
            }
        }
    }
}

impl<T: fmt::Display + Debug + Clone + 'static> fmt::Display for ValueRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRefMut::CellBorrow { value, .. } => std::fmt::Display::fmt(value.as_ref().unwrap(), f),
            ValueRefMut::Borrow(value) => std::fmt::Display::fmt(value.as_ref().unwrap(), f),
            ValueRefMut::Locked(value) => std::fmt::Display::fmt(value.as_ref().unwrap(), f),
            ValueRefMut::TupleState(_, value) => std::fmt::Display::fmt(value.as_ref().unwrap(), f),
            ValueRefMut::Read(val) => std::fmt::Display::fmt(val, f),
        }
    }
}

impl<T: fmt::Debug + Clone + 'static> fmt::Debug for ValueRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRefMut::CellBorrow { value, .. } => {
                f.debug_struct("ValueRefMut::CellBorrow")
                    .field("value", &value.as_ref().unwrap())
                    .finish()
            },
            ValueRefMut::Borrow(value) => {
                f.debug_struct("ValueRefMut::Borrow")
                    .field("value", &value.as_ref().unwrap())
                    .finish()
            },
            ValueRefMut::Locked(value) => {
                f.debug_struct("ValueRefMut::Locked")
                    .field("value", &value.as_ref().unwrap())
                    .finish()
            },
            ValueRefMut::TupleState(_, value) => {
                f.debug_struct("ValueRefMut::TupleState")
                    .field("value", &value.as_ref().unwrap())
                    .finish()
            }
            ValueRefMut::Read(val) => {
                fmt::Debug::fmt(val, f)
            }
        }
    }
}