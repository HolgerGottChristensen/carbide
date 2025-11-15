use std::fmt::Debug;
use carbide_core::state::{IntoReadState, ReadState};
use chrono::{DateTime, FixedOffset, TimeZone};
use fluent_for_carbide::types::{FluentDateTimeOptions, FluentNumberOptions};
use carbide_core::impl_state_value;

pub trait Args: Debug + Clone + 'static {
    fn push<T: IntoReadState<G>, G: Arg>(self, key: &'static str, value: T) -> impl Args;

    fn iter(&self) -> impl Iterator<Item=(&str, LocalizedArg)>;
}

impl Args for () {
    fn push<T: IntoReadState<G>, G: Arg>(self, key: &'static str, value: T) -> impl Args {
        LocalizedArgList {
            key,
            value: value.into_read_state(),
            rest: (),
        }
    }

    fn iter(&self) -> impl Iterator<Item=(&str, LocalizedArg)> {
        std::iter::empty()
    }
}

impl<S: ReadState<T=V>, V: Arg, R: Args> Args for LocalizedArgList<S, V, R> {
    fn push<T: IntoReadState<G>, G: Arg>(self, key: &'static str, value: T) -> impl Args {
        LocalizedArgList {
            key,
            value: value.into_read_state(),
            rest: self,
        }
    }

    fn iter(&self) -> impl Iterator<Item=(&str, LocalizedArg)> {
        let val = self.value.value();
        let arg = Arg::into(&*val);
        std::iter::once((self.key, arg)).chain(self.rest.iter())
    }
}


pub trait Arg: Clone + Debug + 'static {
    fn into(&self) -> LocalizedArg;
}

impl Arg for LocalizedArg {
    fn into(&self) -> LocalizedArg {
        self.clone()
    }
}

impl Arg for f64 {
    fn into(&self) -> LocalizedArg {
        LocalizedArg::Number(*self, FluentNumberOptions::default())
    }
}

impl Arg for i32 {
    fn into(&self) -> LocalizedArg {
        LocalizedArg::Number(*self as f64, FluentNumberOptions::default())
    }
}

impl Arg for &'static str {
    fn into(&self) -> LocalizedArg {
        LocalizedArg::Str(*self)
    }
}

impl Arg for String {
    fn into(&self) -> LocalizedArg {
        LocalizedArg::String(self.clone())
    }
}

impl<Tz: TimeZone + 'static> Arg for DateTime<Tz> {
    fn into(&self) -> LocalizedArg {
        LocalizedArg::Date(self.fixed_offset(), FluentDateTimeOptions::default())
    }
}

#[derive(Clone, Debug)]
pub enum LocalizedArg {
    Str(&'static str),
    String(String),
    Number(f64, FluentNumberOptions),
    Date(DateTime<FixedOffset>, FluentDateTimeOptions),
}

#[derive(Debug, Clone)]
pub struct LocalizedArgList<S: ReadState<T=V>, V: Arg, R: Args> {
    key: &'static str,
    value: S,
    rest: R,
}

impl_state_value!(LocalizedArg);