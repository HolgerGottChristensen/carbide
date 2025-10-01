use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use crate::draw::{Angle, Color, Dimension, ImageId, Position, Rect};
use crate::environment::{Environment, EnvironmentColor, EnvironmentFontSize};
use crate::focus::Focus;
use crate::render::Style;
use crate::state::state_sync::StateSync;
use crate::state::util::value_cell::ValueRef;
use crate::state::*;
use crate::widget::{EdgeInsets};
use crate::math::{Matrix2, Matrix3, Matrix4, Vector1, Vector2, Vector3, Vector4};
use dyn_clone::{clone_box, DynClone};
use crate::draw::gradient::Gradient;
// ---------------------------------------------------
//  Definitions
// ---------------------------------------------------

pub trait ReadState: AnyReadState + Clone + IntoReadState<Self::T> + private::Sealed {
    /// This retrieves a immutable reference to the value contained in the state.
    /// This type implements deref to get a reference to the actual value. The [`ValueRef`]
    /// should not be used directly.
    fn value(&self) -> ValueRef<'_, Self::T>;
}

/// The trait to implement for read-only state.
pub trait AnyReadState: DynClone + StateSync + Debug + 'static {
    type T: StateContract;

    fn value_dyn(&self) -> ValueRef<'_, Self::T>;
}

impl<T: StateContract> dyn AnyReadState<T=T> {
    pub fn boxed(&self) -> Box<dyn AnyReadState<T=T>> {
        clone_box(self)
    }
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

    fn value_dyn(&self) -> ValueRef<'_, Self::T> {
        self.deref().value_dyn()
    }
}

// Blanket implementation, implementing ReadState for all types that
// implement AnyReadState, Clone and IntoReadState
impl<T> ReadState for T where T: AnyReadState + Clone + IntoReadState<Self::T> {
    fn value(&self) -> ValueRef<'_, Self::T> {
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
    fn value_dyn(&self) -> ValueRef<'_, G> {
        self.deref().value()
    }
}

impl<T, E> StateSync for Result<T, E> {}
impl<T: Debug + Clone + 'static, E: Debug + Clone + 'static> AnyReadState for Result<T, E> {
    type T = Result<T, E>;
    fn value_dyn(&self) -> ValueRef<'_, Result<T, E>> {
        ValueRef::Borrow(self)
    }
}
impl<T: Debug + Clone + 'static, E: Debug + Clone + 'static> AnyState for Result<T, E> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<'_, Result<T, E>> {
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
        impl $crate::state::StateSync for $typ {
            fn sync(&mut self, _env: &mut $crate::environment::Environment) -> bool {
                true
            }
        }
        impl $crate::state::AnyReadState for $typ {
            type T = $typ;
            fn value_dyn(&self) -> $crate::state::ValueRef<'_, $typ> {
                $crate::state::ValueRef::Borrow(self)
            }
        }
        impl $crate::state::AnyState for $typ {
            fn value_dyn_mut(&mut self) -> $crate::state::ValueRefMut<'_, $typ> {
                $crate::state::ValueRefMut::Borrow(Some(self))
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
    ImageId
);

#[macro_export]
macro_rules! impl_state_value_generic {
    ($($typ: ident < $( $generic: ident $(: $bound: ident $(+ $rest: ident)*)? ),* > ),*) => {
        $(
        impl<$($generic $(: $bound $(+ $rest)* )?),*> $crate::state::StateSync for $typ <$($generic),*> {}
        impl<$($generic: std::fmt::Debug + Clone + 'static $(+ $bound $(+ $rest)* )?),*> $crate::state::AnyReadState for $typ<$($generic),*> {
            type T = $typ<$($generic),*>;
            fn value_dyn(&self) -> $crate::state::ValueRef<'_, $typ<$($generic),*>> {
                $crate::state::ValueRef::Borrow(self)
            }
        }
        impl<$($generic: std::fmt::Debug + Clone + 'static $(+ $bound $(+ $rest)* )?),*> $crate::state::AnyState for $typ <$($generic),*> {
            fn value_dyn_mut(&mut self) -> $crate::state::ValueRefMut<'_, $typ<$($generic),*>> {
                $crate::state::ValueRefMut::Borrow(Some(self))
            }

            fn set_value_dyn(&mut self, value: $typ<$($generic),*>) {
                *self = value;
            }
        }
        )*
    };
}

impl_state_value_generic!(
    Option<T>, Vec<T>, Matrix4<T>,
    Matrix3<T>, Matrix2<T>, Vector1<T>,
    Vector2<T>, Vector3<T>, Vector4<T>
);