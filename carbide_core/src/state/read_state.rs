use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use cgmath::Matrix4;

use dyn_clone::DynClone;
use carbide_core::environment::Environment;
use crate::Color;
use crate::focus::Focus;
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

impl<G: NewStateSync> NewStateSync for Box<G> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.deref_mut().sync(env)
    }
}
impl<G: Debug + Clone + 'static, F: ReadState<T=G> + Clone> ReadState for Box<F> {
    type T = G;
    fn value(&self) -> ValueRef<G> {
        self.deref().value()
    }
}

macro_rules! impl_read_state {
    ($($typ: ty),*) => {
        $(
        impl NewStateSync for $typ {}
        impl ReadState for $typ {
            type T = $typ;
            fn value(&self) -> ValueRef<$typ> {
                ValueRef::Borrow(self)
            }
        }
        impl State for $typ {
            fn value_mut(&mut self) -> ValueRefMut<$typ> {
                ValueRefMut::Borrow(self)
            }

            fn set_value(&mut self, value: $typ) {
                *self = value;
            }
        }
        )*

    };
}

impl_read_state!(
    i8, u8, i16, u16,
    i32, u32, i64, u64,
    i128, u128, f32, f64,
    bool, char, isize, usize,
    Style, String, (), Color, &'static str, Focus
);