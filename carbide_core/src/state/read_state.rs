use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use cgmath::Matrix4;

use dyn_clone::DynClone;
use carbide_core::environment::Environment;
use crate::Color;
use crate::color::{BLACK, ORANGE};
use crate::environment::{EnvironmentColor, EnvironmentFontSize};
use crate::focus::Focus;
use crate::render::Style;

use crate::state::*;
use crate::state::state_sync::NewStateSync;
use crate::state::util::value_cell::ValueRef;

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=ec8e5b8920ccb55c4d552666fe7f1179


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

pub trait IntoReadState<T>: Clone where T: StateContract {
    type Output: ReadState<T=T>;

    fn into_read_state(self) -> Self::Output;
}

pub trait IntoReadStateHelper<T, U, B: StateContract>: Clone where T: AnyReadState<T=U> {
    type Output: ReadState<T=B>;

    fn into_read_state_helper(self) -> Self::Output;
}


mod private {
    use crate::state::AnyReadState;

    pub trait Sealed {}

    impl<T> Sealed for T where T: AnyReadState {}
}

impl<T> ReadState for T where T: AnyReadState + Clone + IntoReadState<Self::T> {
    fn value(&self) -> ValueRef<Self::T> {
        self.value_dyn()
    }
}

impl<T> IntoReadStateHelper<T, Color, Style> for T where T: AnyReadState<T=Color> + Clone {
    type Output = RMap1<fn(&Color)->Style, Color, Style, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map(self, |c| {
            Style::Color(*c)
        })
    }
}

impl<T> IntoReadStateHelper<T, EnvironmentColor, Color> for T where T: AnyReadState<T=EnvironmentColor> + Clone {
    type Output = EnvMap1<fn(&Environment, &EnvironmentColor)->Color, EnvironmentColor, Color, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map_env(self, |env, value| {
            env.get_color(&StateKey::Color(value.clone())).unwrap()
        })
    }
}

impl<T> IntoReadStateHelper<T, EnvironmentColor, Style> for T where T: AnyReadState<T=EnvironmentColor> + Clone {
    type Output = EnvMap1<fn(&Environment, &EnvironmentColor)->Style, EnvironmentColor, Style, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map_env(self, |env, value| {
            Style::Color(env.get_color(&StateKey::Color(value.clone())).unwrap())
        })
    }
}

impl<T> IntoReadStateHelper<T, EnvironmentFontSize, u32> for T where T: AnyReadState<T=EnvironmentFontSize> + Clone {
    type Output = EnvMap1<fn(&Environment, &EnvironmentFontSize)->u32, EnvironmentFontSize, u32, T>;

    fn into_read_state_helper(self) -> Self::Output {
        Map1::read_map_env(self, |env, value| {
            env.get_font_size(&StateKey::FontSize(value.clone())).unwrap()
        })
    }
}

impl<T, U: StateContract> IntoReadStateHelper<T, U, U> for T where T: AnyReadState<T=U> + Clone {
    type Output = T;

    fn into_read_state_helper(self) -> Self::Output {
        self
    }
}

impl<T: AnyReadState<T=A>, A, B: StateContract> IntoReadState<B> for T where T: IntoReadStateHelper<T, A, B> {
    type Output = T::Output;

    fn into_read_state(self) -> Self::Output {
        self.into_read_state_helper()
    }
}

/*impl<T: StateContract, U> IntoReadState<T> for U where U: AnyReadState<T=T> + Clone {
    type Output = U;

    fn into_read_state(self) -> Self::Output {
        self
    }
}*/

dyn_clone::clone_trait_object!(<T: StateContract> AnyReadState<T=T>);

impl<G> NewStateSync for Vec<G> {}
impl<G: Debug + Clone + 'static> AnyReadState for Vec<G> {
    type T = Vec<G>;
    fn value_dyn(&self) -> ValueRef<Vec<G>> {
        ValueRef::Borrow(self)
    }
}
impl<G: Debug + Clone + 'static> AnyState for Vec<G> {
    fn value_dyn_mut(&mut self) -> ValueRefMut<Vec<G>> {
        ValueRefMut::Borrow(self)
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

macro_rules! impl_read_state {
    ($($typ: ty),*) => {
        $(
        impl NewStateSync for $typ {
            fn sync(&mut self, env: &mut Environment) -> bool {
                true
            }
        }
        impl AnyReadState for $typ {
            type T = $typ;
            fn value_dyn(&self) -> ValueRef<$typ> {
                ValueRef::Borrow(self)
            }
        }
        impl AnyState for $typ {
            fn value_dyn_mut(&mut self) -> ValueRefMut<$typ> {
                ValueRefMut::Borrow(self)
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
    Style, String, (), Color, &'static str, Focus, EnvironmentColor, EnvironmentFontSize
);