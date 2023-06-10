use carbide_core::state::ReadState;
use crate::state::{Map1, RMap1, State, StateContract};

pub trait IntoReadState<T> where T: StateContract {
    type Output: ReadState<T=T> + Clone;

    fn into_read_state(self) -> Self::Output;
}

pub trait IntoState<T> where T: StateContract {
    type Output: State<T=T> + Clone;

    fn into_state(self) -> Self::Output;
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
