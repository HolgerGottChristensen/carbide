use std::fmt::Debug;
use std::ops::{Range, RangeInclusive};
use crate::state::{Map1, RMap1, AnyReadState, StateContract, State, AnyState, RWMap1, RWOMap1, ValueRefMut, ValueState, LocalState};


// ---------------------------------------------------
//  Definitions
// ---------------------------------------------------

pub trait IntoState<T> where T: StateContract {
    type Output: State<T=T>;

    fn into_state(self) -> Self::Output;
}

pub trait ConvertInto<T: StateContract>: StateContract {
    type Output<G: AnyState<T=Self> + Clone>: State<T=T>;

    fn convert<F: AnyState<T=Self> + Clone>(f: F) -> Self::Output<F>;
}

// ---------------------------------------------------
//  Implementations
// ---------------------------------------------------

impl<T: StateContract> ConvertInto<T> for T {
    type Output<G: AnyState<T=Self> + Clone> = G;

    fn convert<F: AnyState<T=T> + Clone>(f: F) -> Self::Output<F> {
        f
    }
}

impl<T: AnyState<T=A> + Clone, A: StateContract, B: StateContract> IntoState<B> for T where A: ConvertInto<B> {
    type Output = A::Output<T>;

    fn into_state(self) -> Self::Output {
        A::convert(self)
    }
}

impl<T: StateContract> IntoState<Vec<<Range<T> as Iterator>::Item>> for Range<T> where Range<T>: Iterator, <Range<T> as Iterator>::Item: StateContract {
    type Output = LocalState<Vec<<Range<T> as Iterator>::Item>>;

    fn into_state(self) -> Self::Output {
        LocalState::new(self.collect::<Vec<_>>())
    }
}

impl<T: StateContract> IntoState<Vec<<RangeInclusive<T> as Iterator>::Item>> for RangeInclusive<T> where RangeInclusive<T>: Iterator, <RangeInclusive<T> as Iterator>::Item: StateContract{
    type Output = LocalState<Vec<<RangeInclusive<T> as Iterator>::Item>>;

    fn into_state(self) -> Self::Output {
        LocalState::new(self.collect::<Vec<_>>())
    }
}


/*impl<T: ReadState<T=T> + Clone> IntoReadState<T> for T {
    type Output = T;

    fn into_read_state(self) -> Self::Output {
        self
    }
}*/

macro_rules! impl_string_state {
    ($($typ: ty),*) => {
        $(
        impl carbide_core::state::ConvertIntoRead<String> for $typ {
            type Output<G: AnyReadState<T=Self> + Clone> = RMap1<fn(&$typ)->String, $typ, String, G>;

            fn convert<F: AnyReadState<T=$typ> + Clone>(f: F) -> Self::Output<F> {
                Map1::read_map(f, |s| {
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


impl ConvertInto<Result<String, String>> for String {
    type Output<G: AnyState<T=Self> + Clone> = RWMap1<fn(&String) -> Result<String, String>, fn(Result<String, String>, ValueRefMut<String>), String, Result<String, String>, G>;

    fn convert<F: AnyState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        Map1::map(f, |val| {
            Ok(val.to_string())
        }, |new, mut value| {
            match new {
                Ok(s) | Err(s) => {
                    *value = s;
                }
            }
        })
    }
}

macro_rules! impl_res_state_plain {
    ($($typ: ty),*) => {
        $(
            impl ConvertInto<Result<String, String>> for Result<$typ, String> {
                type Output<G: AnyState<T=Self> + Clone> = RWMap1<fn(&Result<$typ, String>)->Result<String, String>, fn(Result<String, String>, ValueRefMut<Result<$typ, String>>), Result<$typ, String>, Result<String, String>, G>;

                fn convert<F: AnyState<T=Self> + Clone>(f: F) -> Self::Output<F> {
                    use std::str::FromStr;

                    Map1::map(f, |val| {
                        match val {
                            Ok(val) => { Ok(val.to_string()) }
                            Err(val) => { Err(val.to_string()) }
                        }
                    }, |new, mut val| {
                        match new {
                            Ok(s) | Err(s) => {
                                *val = <$typ>::from_str(&s).map_err(|_| s.to_string())
                            }
                        }
                    })
                }
            }
        )*
    }
}

impl_res_state_plain! {
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize
}


// TODO: We cant do float mappings nicely, because we dont have access to the old value when mapping forwards.
impl ConvertInto<Result<String, String>> for Result<f32, String> {
    type Output<G: AnyState<T=Self> + Clone> = RWMap1<fn(&Result<f32, String>) -> Result<String, String>, fn(Result<String, String>, ValueRefMut<Result<f32, String>>), Result<f32, String>, Result<String, String>, G>;

    fn convert<F: AnyState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        use std::str::FromStr;

        Map1::map(f, |val| {
            match val {
                Ok(val) => { Ok(val.to_string()) }
                Err(val) => { Err(val.to_string()) }
            }
        }, |new, mut val| {
            match new {
                Ok(s) | Err(s) => {
                    *val = <f32>::from_str(&s).map_err(|_| s);
                }
            }
        })
    }
}

impl ConvertInto<Result<String, String>> for Result<f64, String> {
    type Output<G: AnyState<T=Self> + Clone> = RWOMap1<
        fn(&Result<f64, String>, &mut Result<String, String>),
        fn(Result<String, String>, ValueRefMut<Result<f64, String>>),
        Result<f64, String>,
        Result<String, String>,
        G
    >;

    fn convert<F: AnyState<T=Self> + Clone>(f: F) -> Self::Output<F> {
        use std::str::FromStr;

        Map1::map_owned(f, |val, current| {
            match (val, current) {
                (Ok(v), Ok(o)) => {
                    println!("Both ok");
                    if let Ok(k) = <f64>::from_str(o) {
                        if *v != k {
                            *o = v.to_string();
                        } else {
                            println!("Equal");
                        }
                    } else {
                        *o = v.to_string();
                    }
                }
                (e, t) => {
                    *t = e.clone().map(|b| b.to_string());
                }
            }
        }, |new, mut old| {
            match new {
                Ok(s) | Err(s) => {
                    *old = <f64>::from_str(&s).map_err(|_| s);
                }
            }
        }, Err(String::new()))
    }
}




/*impl IntoReadState<f64> for u32 {
    type Output = f64;

    fn into_read_state(self) -> Self::Output {
        self as f64
    }
}



impl<T: StateContract> IntoReadState<T> for T where T: ReadState<T=T> {
    type Output = T;

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
}*/