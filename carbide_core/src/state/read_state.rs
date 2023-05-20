use std::fmt::Debug;
use cgmath::Matrix4;

use dyn_clone::DynClone;
use crate::Color;
use crate::render::Style;

use crate::state::*;
use crate::state::state_sync::NewStateSync;
use crate::state::util::value_cell::ValueRef;

/// The trait to implement for read-only state.
pub trait ReadState: DynClone + NewStateSync + Debug + 'static {
    type T: StateContract;
    /// This retrieves a immutable reference to the value contained in the state.
    /// This type implements deref to get a reference to the actual value. The [`ValueRef`]
    /// should not be used directly.
    fn value(&self) -> ValueRef<Self::T>;
}

dyn_clone::clone_trait_object!(<T: StateContract> ReadState<T=T>);

impl NewStateSync for () {}
impl ReadState for () {
    type T = ();
    fn value(&self) -> ValueRef<()> {
        ValueRef::Borrow(self)
    }
}

impl NewStateSync for bool {}
impl ReadState for bool {
    type T = bool;
    fn value(&self) -> ValueRef<bool> {
        ValueRef::Borrow(self)
    }
}

impl NewStateSync for Color {}
impl ReadState for Color {
    type T = Color;
    fn value(&self) -> ValueRef<Color> {
        ValueRef::Borrow(self)
    }
}

impl NewStateSync for usize {}
impl ReadState for usize {
    type T = usize;
    fn value(&self) -> ValueRef<usize> {
        ValueRef::Borrow(self)
    }
}

impl<G> NewStateSync for Matrix4<G> {}
impl<G: Debug + Clone + 'static> ReadState for Matrix4<G> {
    type T = Matrix4<G>;
    fn value(&self) -> ValueRef<Matrix4<G>> {
        ValueRef::Borrow(self)
    }
}

impl<G> NewStateSync for Option<G> {}
impl<G: Debug + Clone + 'static> ReadState for Option<G> {
    type T = Option<G>;
    fn value(&self) -> ValueRef<Option<G>> {
        ValueRef::Borrow(self)
    }
}

impl<G> NewStateSync for Vec<G> {}
impl<G: Debug + Clone + 'static> ReadState for Vec<G> {
    type T = Vec<G>;
    fn value(&self) -> ValueRef<Vec<G>> {
        ValueRef::Borrow(self)
    }
}
impl<G: Debug + Clone + 'static> State for Vec<G> {
    fn value_mut(&mut self) -> ValueRefMut<Vec<G>> {
        ValueRefMut::Borrow(self)
    }

    fn set_value(&mut self, value: Vec<G>) {
        *self = value;
    }
}

impl NewStateSync for Style {}
impl ReadState for Style {
    type T = Style;
    fn value(&self) -> ValueRef<Style> {
        ValueRef::Borrow(self)
    }
}

impl NewStateSync for u32 {}
impl ReadState for u32 {
    type T = u32;
    fn value(&self) -> ValueRef<u32> {
        ValueRef::Borrow(self)
    }
}

impl NewStateSync for String {}
impl ReadState for String {
    type T = String;
    fn value(&self) -> ValueRef<String> {
        ValueRef::Borrow(self)
    }
}