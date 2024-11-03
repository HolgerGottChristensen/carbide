use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use cgmath::{Matrix2, Matrix3, Matrix4, Vector1, Vector2, Vector3, Vector4};
use dyn_clone::DynClone;
use carbide::text::FontWeight;

use carbide_core::environment::Environment;

use crate::draw::{Angle, Color, Dimension, Position, Rect, ImageId};
use crate::environment::{EnvironmentColor, EnvironmentFontSize};
use crate::focus::Focus;
use crate::render::Style;
use crate::state::*;
use crate::state::state_sync::StateSync;
use crate::state::util::value_cell::ValueRef;
use crate::text::FontStyle;
use crate::widget::{EdgeInsets, Gradient};

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
pub trait AnyReadState: DynClone + StateSync + Debug + 'static {
    type T: StateContract;

    fn value_dyn(&self) -> ValueRef<Self::T>;
}

// ---------------------------------------------------
//  Implementations
// ---------------------------------------------------

impl<T: StateContract> StateSync for Box<dyn AnyReadState<T=T>> {
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














impl<G: StateSync> StateSync for Box<G> {
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

impl<T, E> StateSync for Result<T, E> {}
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
macro_rules! impl_state_value {
    ($($typ: ty),*) => {
        $(
        impl carbide_core::state::StateSync for $typ {
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

impl_state_value!(
    i8, u8, i16, u16,
    i32, u32, i64, u64,
    i128, u128, f32, f64,
    bool, char, isize, usize,
    Style, String, (), Color, &'static str, Focus,
    EnvironmentColor, EnvironmentFontSize, Gradient,
    EdgeInsets, Position, Dimension, Rect, Angle,
    FontStyle, FontWeight, ImageId
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

#[macro_export]
macro_rules! impl_read_state1 {
    ($($typ: ident),*) => {
        $(
        impl<G> StateSync for $typ<G> {}
        impl<G: Debug + Clone + 'static> AnyReadState for $typ<G> {
            type T = $typ<G>;
            fn value_dyn(&self) -> ValueRef<$typ<G>> {
                ValueRef::Borrow(self)
            }
        }
        impl<G: Debug + Clone + 'static> AnyState for $typ<G> {
            fn value_dyn_mut(&mut self) -> ValueRefMut<$typ<G>> {
                ValueRefMut::Borrow(Some(self))
            }

            fn set_value_dyn(&mut self, value: $typ<G>) {
                *self = value;
            }
        }
        )*
    };
}

impl_read_state1!(Option, Vec, Matrix4, Matrix3, Matrix2, Vector1, Vector2, Vector3, Vector4);