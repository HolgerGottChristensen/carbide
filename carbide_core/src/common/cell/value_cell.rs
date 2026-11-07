#![allow(unsafe_code)]

use std::any::Any;
use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Formatter};
use dyn_clone::DynClone;
use crate::cell::{ValueRef, ValueRefMut};

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

pub trait Value: DynClone + Any {}
dyn_clone::clone_trait_object!(Value);

impl<T: Clone + Any> Value for T {}

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