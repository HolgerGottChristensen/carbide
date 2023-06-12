use carbide_core::state::{ReadState, TState};
use crate::draw::draw_gradient::DrawGradient;
use crate::draw::{Dimension, Position};
use crate::focus::Focus;
use crate::render::Style;
use crate::state::{Map1, RMap1, State, StateContract};
use crate::widget::Gradient;

pub trait IntoReadState<T>: Clone where T: StateContract {
    type Output: ReadState<T=T> + Clone;

    fn into_read_state(self) -> Self::Output;
}

pub trait IntoState<T> where T: StateContract {
    type Output: State<T=T> + Clone;

    fn into_state(self) -> Self::Output;
}

impl<T: ReadState<T=T> + Clone> IntoReadState<T> for T {
    type Output = T;

    fn into_read_state(self) -> Self::Output {
        self
    }
}

macro_rules! impl_string_state {
    ($($typ: ty),*) => {
        $(
        impl IntoReadState<String> for $typ {
            type Output = RMap1<fn(&$typ) -> String, $typ, String, $typ>;

            fn into_read_state(self) -> Self::Output {
                Map1::read_map(self, |s| {
                    s.to_string()
                })
            }
        }
        )*

    };
}

impl_string_state!(
    i8, u8, i16, u16,
    i32, u32, i64, u64,
    i128, u128, f32, f64,
    bool, char, isize, usize,
    &'static str
);

impl IntoReadState<f64> for u32 {
    type Output = f64;

    fn into_read_state(self) -> Self::Output {
        self as f64
    }
}



impl<T: StateContract> IntoReadState<T> for TState<T> {
    type Output = TState<T>;

    fn into_read_state(self) -> Self::Output {
        self
    }
}

impl<T: StateContract> IntoState<T> for TState<T> {
    type Output = TState<T>;

    fn into_state(self) -> Self::Output {
        self
    }
}

impl IntoReadState<Style> for Gradient {
    type Output = Style;

    fn into_read_state(self) -> Self::Output {
        Style::Gradient(self)
    }
}