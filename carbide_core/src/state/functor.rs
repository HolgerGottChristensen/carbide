use crate::render::Style;
use crate::draw::Color;
use crate::focus::Focus;
use crate::environment::EnvironmentColor;
use crate::environment::EnvironmentFontSize;
use crate::widget::EdgeInsets;
use crate::draw::Position;
use crate::draw::Dimension;
use crate::draw::Rect;
use crate::draw::Angle;
use crate::draw::gradient::Gradient;
use crate::state::StateContract;
use crate::text::FontStyle;
use crate::text::FontWeight;

pub trait Fn2<I, O>: Fn(&I)->O + Clone + 'static {}

impl<T, I, O> Fn2<I, O> for T where T: Fn(&I)->O + Clone + 'static {}

/// A trait indicating that a type is mappable. This is useful when you want to
/// implement a common method for all wrappers of a certain type.
/// In carbide, this is usually different kinds of State.
pub trait Functor<T: StateContract> {
    type Output<G: StateContract, F: Fn2<T, G>>;

    fn map<U: StateContract, F: Fn2<T, U>>(self, f: F) -> Self::Output<U, F>;
}

#[macro_export]
macro_rules! impl_functor {
    ($($typ: ty),*) => {
        $(
        impl $crate::state::Functor<$typ> for $typ {
            type Output<G: StateContract, F: Fn2<$typ, G>> = G;
            fn map<U: StateContract, F: Fn2<$typ, U>>(self, f: F) -> Self::Output<U, F> {
                f(&self)
            }
        }
        )*
    };
}

impl_functor!(
    i8, u8, i16, u16,
    i32, u32, i64, u64,
    i128, u128, f32, f64,
    bool, char, isize, usize,
    Style, String, (), Color, &'static str, Focus,
    EnvironmentColor, EnvironmentFontSize, Gradient,
    EdgeInsets, Position, Dimension, Rect, Angle,
    FontStyle, FontWeight
);