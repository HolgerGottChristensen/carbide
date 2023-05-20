use carbide_core::state::ReadState;
use crate::state::{Map1, RMap1, State, StateContract};

pub trait IntoState<T> where T: StateContract {
    type Output: ReadState<T=T> + Clone;

    fn into_state(self) -> Self::Output;
}

impl<T, G: ToString + StateContract> IntoState<String> for T where T: ReadState<T=G> + Clone {
    type Output = RMap1<fn(&G) -> String, G, String, T>;

    fn into_state(self) -> Self::Output {
        Map1::read_map(self, |s| {
            s.to_string()
        })
    }
}


/*macro_rules! impl_string_state {
    ($($typ: ty),*) => {
        $(
        impl<T> IntoState<String> for T where T: ReadState<T=$typ> + Clone {
            type Output = RMap1<fn(&$typ) -> String, $typ, String, T>;

            fn into_state(self) -> Self::Output {
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
    bool, char, isize, usize
);*/