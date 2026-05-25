use std::fmt;
use std::ops::Deref;
use parking_lot::MappedRwLockReadGuard;
use carbide::common::cell::value_cell::BorrowRef;

pub type Ref<'a, T: 'a> = ValueRef<'a, T>;

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