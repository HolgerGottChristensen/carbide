#![allow(unsafe_code)]

use std::cell::{Cell, UnsafeCell};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

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

impl<T: Debug> Debug for ValueCell<T> {
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

impl<T> ValueCell<T> {
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

    pub fn borrow_mut(&self) -> ValueRefMut<'_, T> {
        self.try_borrow_mut().expect("Already borrowed")
    }

    pub fn try_borrow_mut(&self) -> Result<ValueRefMut<'_, T>, ()> {
        match BorrowRefMut::new(&self.borrow) {
            // SAFETY: `BorrowRef` guarantees unique access.
            Some(b) => Ok(ValueRefMut::CellBorrow {
                value: unsafe { &mut *self.value.get() },
                borrow: b,
            }),
            None => Err(()),
        }
    }
}

unsafe impl<T: ?Sized> Send for ValueCell<T> where T: Send {}

impl<T: Clone> Clone for ValueCell<T> {
    fn clone(&self) -> ValueCell<T> {
        ValueCell::new(self.borrow().clone())
    }
}

impl<T: Default> Default for ValueCell<T> {
    fn default() -> ValueCell<T> {
        ValueCell::new(Default::default())
    }
}

impl<T: PartialEq> PartialEq for ValueCell<T> {
    fn eq(&self, other: &ValueCell<T>) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: PartialOrd> PartialOrd for ValueCell<T> {
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
        }
    }
}

impl<'b, T: Clone> ValueRef<'b, T> {
    #[inline]
    pub fn clone(orig: &ValueRef<'b, T>) -> ValueRef<'b, T> {
        match orig {
            ValueRef::CellBorrow { value, borrow } => ValueRef::CellBorrow {
                value: *value,
                borrow: borrow.clone(),
            },
            ValueRef::Borrow(value) => ValueRef::Borrow(*value),
            ValueRef::Owned(value) => ValueRef::Owned(value.clone()),
        }
    }

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
        }
    }
}

impl<T: fmt::Display> fmt::Display for ValueRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRef::CellBorrow { value, .. } => value.fmt(f),
            ValueRef::Borrow(value) => value.fmt(f),
            ValueRef::Owned(value) => value.fmt(f),
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

pub enum ValueRefMut<'a, T: 'a> {
    CellBorrow {
        value: &'a mut T,
        borrow: BorrowRefMut<'a>,
    },
    Borrow(&'a mut T),
    Owned(T),
}

impl<'b, T: Clone> ValueRefMut<'b, T> {
    #[inline]
    pub fn map<U: Clone, F>(orig: ValueRefMut<'b, T>, f: F) -> ValueRefMut<'b, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        match orig {
            ValueRefMut::CellBorrow { value, borrow } => ValueRefMut::CellBorrow {
                value: f(value),
                borrow,
            },
            ValueRefMut::Borrow(value) => ValueRefMut::Borrow(f(value)),
            ValueRefMut::Owned(mut value) => ValueRefMut::Owned(f(&mut value).clone())
        }
    }
}

impl<T> Deref for ValueRefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        match self {
            ValueRefMut::CellBorrow { value, .. } => *value,
            ValueRefMut::Borrow(value) => *value,
            ValueRefMut::Owned(value) => value,
        }
    }
}

impl<T> DerefMut for ValueRefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        match self {
            ValueRefMut::CellBorrow { value, .. } => *value,
            ValueRefMut::Borrow(value) => *value,
            ValueRefMut::Owned(value) => value,
        }
    }
}

impl<T: fmt::Display> fmt::Display for ValueRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRefMut::CellBorrow { value, .. } => value.fmt(f),
            ValueRefMut::Borrow(value) => value.fmt(f),
            ValueRefMut::Owned(value) => value.fmt(f),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for ValueRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueRefMut::CellBorrow { value, .. } => value.fmt(f),
            ValueRefMut::Borrow(value) => value.fmt(f),
            ValueRefMut::Owned(value) => value.fmt(f),
        }
    }
}
