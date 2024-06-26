use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use cgmath::Matrix4;

use dyn_clone::DynClone;
use carbide_core::environment::Environment;
use crate::draw::{Color, Rect};
use crate::environment::{EnvironmentColor, EnvironmentFontSize};
use crate::focus::Focus;
use crate::render::Style;

use crate::state::*;
use crate::state::state_sync::NewStateSync;
use crate::state::util::value_cell::ValueRef;
use crate::widget::{EdgeInsets, Gradient};
use crate::draw::{Position, Dimension};

// ---------------------------------------------------
//  Definitions
// ---------------------------------------------------

pub trait ReadState: AnyReadState + Clone + IntoReadState<Self::T> + private::Sealed {
    /// This retrieves a immutable reference to the value contained in the state.
    /// This type implements deref to get a reference to the actual value. The [`ValueRef`]
    /// should not be used directly.
    fn value(&self) -> ValueRef<Self::T>;
}

/// The trait to implement for read-only state.
pub trait AnyReadState: DynClone + NewStateSync + Debug + 'static {
    type T: StateContract;

    fn value_dyn(&self) -> ValueRef<Self::T>;
}

// ---------------------------------------------------
//  Implementations
// ---------------------------------------------------

impl<T: StateContract> NewStateSync for Box<dyn AnyReadState<T=T>> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.deref_mut().sync(env)
    }
}

impl<T: StateContract> AnyReadState for Box<dyn AnyReadState<T=T>> {
    type T = T;

    fn value_dyn(&self) -> ValueRef<Self::T> {
        self.deref().value_dyn()
    }
}

// Blanket implementation, implementing ReadState for all types that
// implement AnyReadState, Clone and IntoReadState
impl<T> ReadState for T where T: AnyReadState + Clone + IntoReadState<Self::T> {
    fn value(&self) -> ValueRef<Self::T> {
        self.value_dyn()
    }
}

dyn_clone::clone_trait_object!(<T: StateContract> AnyReadState<T=T>);

// ---------------------------------------------------
//  Utility
// ---------------------------------------------------

mod private {
    use crate::state::AnyReadState;

    // This disallows implementing ReadState manually, and requires something to implement
    // AnyReadState to implement ReadState.
    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyReadState {}
}
















impl<G> NewStateSync for Vec<G> {}
impl<G: Debug + Clone + 'static> AnyReadState for Vec<G> {
    type T = Vec<G>;
    fn value_dyn(&self) -> ValueRef<Vec<G>> {
        ValueRef::Borrow(self)
    }
}
impl<G: Debug + Clone + 'static> AnyState for Vec<G> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<Vec<G>> {
        ValueRefMut::Borrow(Some(self))
    }

    fn set_value_dyn(&mut self, value: Vec<G>) {
        *self = value;
    }
}


impl<G> NewStateSync for Matrix4<G> {}
impl<G: Debug + Clone + 'static> AnyReadState for Matrix4<G> {
    type T = Matrix4<G>;
    fn value_dyn(&self) -> ValueRef<Matrix4<G>> {
        ValueRef::Borrow(self)
    }
}

impl<G> NewStateSync for Option<G> {}
impl<G: Debug + Clone + 'static> AnyReadState for Option<G> {
    type T = Option<G>;
    fn value_dyn(&self) -> ValueRef<Option<G>> {
        ValueRef::Borrow(self)
    }
}
impl<G: Debug + Clone + 'static> AnyState for Option<G> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<Option<G>> {
        ValueRefMut::Borrow(Some(self))
    }

    fn set_value_dyn(&mut self, value: Option<G>) {
        *self = value;
    }
}

impl<G: NewStateSync> NewStateSync for Box<G> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.deref_mut().sync(env)
    }
}
impl<G: Debug + Clone + 'static, F: ReadState<T=G> + Clone> AnyReadState for Box<F> {
    type T = G;
    fn value_dyn(&self) -> ValueRef<G> {
        self.deref().value()
    }
}

impl<T, E> NewStateSync for Result<T, E> {}
impl<T: Debug + Clone + 'static, E: Debug + Clone + 'static> AnyReadState for Result<T, E> {
    type T = Result<T, E>;
    fn value_dyn(&self) -> ValueRef<Result<T, E>> {
        ValueRef::Borrow(self)
    }
}
impl<T: Debug + Clone + 'static, E: Debug + Clone + 'static> AnyState for Result<T, E> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<Result<T, E>> {
        ValueRefMut::Borrow(Some(self))
    }

    fn set_value_dyn(&mut self, value: Result<T, E>) {
        *self = value;
    }
}

#[macro_export]
macro_rules! impl_read_state {
    ($($typ: ty),*) => {
        $(
        impl carbide_core::state::NewStateSync for $typ {
            fn sync(&mut self, _env: &mut carbide_core::environment::Environment) -> bool {
                true
            }
        }
        impl carbide_core::state::AnyReadState for $typ {
            type T = $typ;
            fn value_dyn(&self) -> carbide_core::state::ValueRef<$typ> {
                carbide_core::state::ValueRef::Borrow(self)
            }
        }
        impl carbide_core::state::AnyState for $typ {
            fn value_dyn_mut(&mut self) -> carbide_core::state::ValueRefMut<$typ> {
                carbide_core::state::ValueRefMut::Borrow(Some(self))
            }

            fn set_value_dyn(&mut self, value: $typ) {
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
    Style, String, (), Color, &'static str, Focus,
    EnvironmentColor, EnvironmentFontSize, Gradient,
    EdgeInsets, Position, Dimension, Rect
);
/*
impl IntoReadStateHelper<i32, i32, u32> for i32 {
    type Output = RMap1<fn(&i32)->u32, i32, u32, i32>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map(self, |c| {
            *c as u32
        })
    }
}

impl IntoReadStateHelper<i32, i32, f64> for i32 {
    type Output = RMap1<fn(&i32)->f64, i32, f64, i32>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map(self, |c| {
            *c as f64
        })
    }
}*/