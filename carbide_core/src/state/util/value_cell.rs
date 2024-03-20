#![allow(unsafe_code)]

use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard};

type BorrowFlag = isize;

const UNUSED: BorrowFlag = 0;

#[inline(always)]
fn is_writing(x: BorrowFlag) -> bool {
    x < UNUSED
}

#[inline(always)]
fn is_reading(x: BorrowFlag) -> bool {
    x > UNUSED
}

pub struct ValueCell<T: ?Sized> {
    borrow: Cell<BorrowFlag>,
    value: UnsafeCell<T>,
}

impl<T: Debug + Clone> Debug for ValueCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InnerState")
            .field("value", &*self.borrow())
            .finish()
    }
}

impl<T> ValueCell<T> {
    pub const fn new(value: T) -> ValueCell<T> {
        ValueCell {
            value: UnsafeCell::new(value),
            borrow: Cell::new(UNUSED),
        }
    }
}

impl<T: Debug + Clone> ValueCell<T> {
    pub fn borrow(&self) -> ValueRef<'_, T> {
        self.try_borrow().expect("Already borrowed")
    }

    pub fn try_borrow(&self) -> Result<ValueRef<'_, T>, ()> {
        match BorrowRef::new(&self.borrow) {
            // SAFETY: `BorrowRef` ensures that there is only immutable access
            // to the value while borrowed.
            Some(b) => Ok(ValueRef::CellBorrow {
                value: unsafe { &*self.value.get() },
                borrow: b,
            }),
            None => Err(()),
        }
    }

    pub fn borrow_mut<'a>(&'a self) -> ValueRefMut<'a, T> {
        self.try_borrow_mut().expect("Already borrowed")
    }

    pub fn try_borrow_mut<'a>(&'a self) -> Result<ValueRefMut<'a, T>, ()> {
        match BorrowRefMut::new(&self.borrow) {
            // SAFETY: `BorrowRef` guarantees unique access.
            Some(b) => Ok(ValueRefMut::CellBorrow {
                value: unsafe { Some(&mut *self.value.get()) },
                borrow: Some(b),
            }),
            None => Err(()),
        }
    }
}

unsafe impl<T: ?Sized> Send for ValueCell<T> where T: Send {}

impl<T: Clone + Debug> Clone for ValueCell<T> {
    fn clone(&self) -> ValueCell<T> {
        ValueCell::new(self.borrow().clone())
    }
}

impl<T: Default> Default for ValueCell<T> {
    fn default() -> ValueCell<T> {
        ValueCell::new(Default::default())
    }
}

impl<T: PartialEq + Debug + Clone> PartialEq for ValueCell<T> {
    fn eq(&self, other: &ValueCell<T>) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: PartialOrd + Debug + Clone> PartialOrd for ValueCell<T> {
    fn partial_cmp(&self, other: &ValueCell<T>) -> Option<Ordering> {
        self.borrow().partial_cmp(&*other.borrow())
    }

    fn lt(&self, other: &ValueCell<T>) -> bool {
        *self.borrow() < *other.borrow()
    }

    fn le(&self, other: &ValueCell<T>) -> bool {
        *self.borrow() <= *other.borrow()
    }

    fn gt(&self, other: &ValueCell<T>) -> bool {
        *self.borrow() > *other.borrow()
    }

    fn ge(&self, other: &ValueCell<T>) -> bool {
        *self.borrow() >= *other.borrow()
    }
}

impl<T> From<T> for ValueCell<T> {
    fn from(t: T) -> ValueCell<T> {
        ValueCell::new(t)
    }
}

pub struct BorrowRef<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl<'b> BorrowRef<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRef<'b>> {
        let b = borrow.get().wrapping_add(1);
        if !is_reading(b) {
            // Incrementing borrow can result in a non-reading value (<= 0) in these cases:
            // 1. It was < 0, i.e. there are writing borrows, so we can't allow a read borrow
            //    due to Rust's reference aliasing rules
            // 2. It was isize::MAX (the max amount of reading borrows) and it overflowed
            //    into isize::MIN (the max amount of writing borrows) so we can't allow
            //    an additional read borrow because isize can't represent so many read borrows
            //    (this can only happen if you mem::forget more than a small constant amount of
            //    `Ref`s, which is not good practice)
            None
        } else {
            // Incrementing borrow can result in a reading value (> 0) in these cases:
            // 1. It was = 0, i.e. it wasn't borrowed, and we are taking the first read borrow
            // 2. It was > 0 and < isize::MAX, i.e. there were read borrows, and isize
            //    is large enough to represent having one more read borrow
            borrow.set(b);
            Some(BorrowRef { borrow })
        }
    }
}

impl Drop for BorrowRef<'_> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(is_reading(borrow));
        self.borrow.set(borrow - 1);
    }
}

impl Clone for BorrowRef<'_> {
    #[inline]
    fn clone(&self) -> Self {
        // Since this Ref exists, we know the borrow flag
        // is a reading borrow.
        let borrow = self.borrow.get();
        debug_assert!(is_reading(borrow));
        // Prevent the borrow counter from overflowing into
        // a writing borrow.
        assert!(borrow != isize::MAX);
        self.borrow.set(borrow + 1);
        BorrowRef {
            borrow: self.borrow,
        }
    }
}

pub enum ValueRef<'a, T: 'a> {
    CellBorrow { value: &'a T, borrow: BorrowRef<'a> },
    Locked(MappedRwLockReadGuard<'a, T>),
    Borrow(&'a T),
    Owned(T),
}

impl<T> Deref for ValueRef<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            ValueRef::CellBorrow { value, .. } => *value,
            ValueRef::Borrow(value) => *value,
            ValueRef::Owned(value) => value,
            ValueRef::Locked(val) => &*val,
        }
    }
}

//impl<'b, T: Clone> ValueRef<'b, T> {
impl<'b, T> ValueRef<'b, T> {
    /*#[inline]
    pub fn clone(orig: &ValueRef<'b, T>) -> ValueRef<'b, T> {
        match orig {
            ValueRef::CellBorrow { value, borrow } => ValueRef::CellBorrow {
                value: *value,
                borrow: borrow.clone(),
            },
            ValueRef::Borrow(value) => ValueRef::Borrow(*value),
            ValueRef::Owned(value) => ValueRef::Owned(value.clone()),
            ValueRef::Locked(val) => {
                todo!()
                //ValueRef::Locked(*val, guard.clone())
            }
        }
    }*/

    #[inline]
    pub fn map<U: Clone, F>(orig: ValueRef<'b, T>, f: F) -> ValueRef<'b, U>
    where
        F: FnOnce(&T) -> &U,
    {
        match orig {
            ValueRef::CellBorrow { value, borrow } => ValueRef::CellBorrow {
                value: f(value),
                borrow,
            },
            ValueRef::Borrow(value) => ValueRef::Borrow(f(value)),
            ValueRef::Owned(value) => ValueRef::Owned(f(&value).clone()),
            ValueRef::Locked(value) => {
                ValueRef::Locked(MappedRwLockReadGuard::map(value, f))
            }
        }
    }

    pub fn apply<U: ?Sized>(self, s: &mut U, f: fn(&'b T, &mut U)) {
        match self {
            ValueRef::CellBorrow { value, .. } => {
                f(value, s)
            }
            ValueRef::Locked(_value) => {
                //MappedRwLockReadGuard::map(value, |a| {f(a, s); a});
                todo!()
            }
            ValueRef::Borrow(value) => {
                f(value, s)
            }
            ValueRef::Owned(_value) => {
                todo!()
                //f(&value, s)
            }
        }
    }
}

impl<T: fmt::Display> fmt::Display for ValueRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRef::CellBorrow { value, .. } => value.fmt(f),
            ValueRef::Borrow(value) => value.fmt(f),
            ValueRef::Owned(value) => value.fmt(f),
            ValueRef::Locked(value) => value.fmt(f),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for ValueRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRef::CellBorrow { value, .. } => value.fmt(f),
            ValueRef::Borrow(value) => value.fmt(f),
            ValueRef::Owned(value) => value.fmt(f),
            ValueRef::Locked(value) => value.fmt(f),
        }
    }
}

pub struct BorrowRefMut<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl Drop for BorrowRefMut<'_> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(is_writing(borrow));
        self.borrow.set(borrow + 1);
    }
}

impl<'b> BorrowRefMut<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRefMut<'b>> {
        // NOTE: Unlike BorrowRefMut::clone, new is called to create the initial
        // mutable reference, and so there must currently be no existing
        // references. Thus, while clone increments the mutable refcount, here
        // we explicitly only allow going from UNUSED to UNUSED - 1.
        match borrow.get() {
            UNUSED => {
                borrow.set(UNUSED - 1);
                Some(BorrowRefMut { borrow })
            }
            _ => None,
        }
    }
}

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
            ValueRefMut::Read(val) => &*val
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