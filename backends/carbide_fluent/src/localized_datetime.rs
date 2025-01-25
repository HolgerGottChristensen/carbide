use crate::locale_ext::LocaleKey;
use crate::localizable::Localizable;
use crate::locale;
use carbide_core::environment::Environment;
use carbide_core::state::{AnyReadState, IntoReadState, ReadState, StateSync, ValueRef, ValueState};
use chrono::{DateTime, FixedOffset};
use fluent::types::{FluentDateTime, FluentDateTimeOptions};
use icu::locid::Locale;
use std::fmt::Debug;

pub type DateStyle = fluent::types::FluentDateStyle;
pub type TimeStyle = fluent::types::FluentTimeStyle;
pub type TimezoneStyle = fluent::types::FluentTimezoneStyle;

#[derive(Debug, Clone)]
pub struct LocalizedDateTime<S, D, T, Tz> where S: ReadState<T=DateTime<FixedOffset>>, D: ReadState<T=DateStyle>, T: ReadState<T=TimeStyle>, Tz: ReadState<T=TimezoneStyle> {
    datetime: S,
    date_style: D,
    time_style: T,
    timezone_style: Tz,
    locale: Locale,
}

impl LocalizedDateTime<ValueState<DateTime<FixedOffset>>, ValueState<DateStyle>, ValueState<TimeStyle>, ValueState<TimezoneStyle>> {
    pub fn new<S: IntoReadState<DateTime<FixedOffset>>>(value: S) -> LocalizedDateTime<S::Output, ValueState<DateStyle>, ValueState<TimeStyle>, ValueState<TimezoneStyle>> {
        LocalizedDateTime {
            datetime: value.into_read_state(),
            date_style: ValueState::new(DateStyle::default()),
            time_style: ValueState::new(TimeStyle::default()),
            locale: locale!("en"),
            timezone_style: ValueState::new(TimezoneStyle::default()),
        }
    }
}

impl<S: ReadState<T=DateTime<FixedOffset>>, D: ReadState<T=DateStyle>, T: ReadState<T=TimeStyle>, Tz: ReadState<T=TimezoneStyle>> LocalizedDateTime<S, D, T, Tz> {
    pub fn date_style<D1: IntoReadState<DateStyle>>(self, date_style: D1) -> LocalizedDateTime<S, D1::Output, T, Tz> {
        LocalizedDateTime {
            datetime: self.datetime,
            date_style: date_style.into_read_state(),
            time_style: self.time_style,
            timezone_style: self.timezone_style,
            locale: self.locale,
        }
    }

    pub fn time_style<T1: IntoReadState<TimeStyle>>(self, time_style: T1) -> LocalizedDateTime<S, D, T1::Output, Tz> {
        LocalizedDateTime {
            datetime: self.datetime,
            date_style: self.date_style,
            time_style: time_style.into_read_state(),
            timezone_style: self.timezone_style,
            locale: self.locale,
        }
    }

    pub fn timezone_style<Tz1: IntoReadState<TimezoneStyle>>(self, timezone_style: Tz1) -> LocalizedDateTime<S, D, T, Tz1::Output> {
        LocalizedDateTime {
            datetime: self.datetime,
            date_style: self.date_style,
            time_style: self.time_style,
            timezone_style: timezone_style.into_read_state(),
            locale: self.locale,
        }
    }
}

impl<S: ReadState<T=DateTime<FixedOffset>>, D: ReadState<T=DateStyle>, T: ReadState<T=TimeStyle>, Tz: ReadState<T=TimezoneStyle>> StateSync for LocalizedDateTime<S, D, T, Tz> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.datetime.sync(env);
        self.date_style.sync(env);
        self.time_style.sync(env);
        self.timezone_style.sync(env);

        if let Some(locale) = env.get::<LocaleKey>() {
            self.locale = locale.clone();
            true
        } else {
            false
        }
    }
}

impl<S: ReadState<T=DateTime<FixedOffset>>, D: ReadState<T=DateStyle>, T: ReadState<T=TimeStyle>, Tz: ReadState<T=TimezoneStyle>> AnyReadState for LocalizedDateTime<S, D, T, Tz> {
    type T = String;

    fn value_dyn(&self) -> ValueRef<Self::T> {
        let value = self.datetime.value();
        let date_style = self.date_style.value();
        let time_style = self.time_style.value();
        let timezone_style = self.timezone_style.value();

        let res = FluentDateTime {
            value: *value,
            options: FluentDateTimeOptions {
                date_style: *date_style,
                time_style: *time_style,
                timezone_style: *timezone_style,
            },
        }.as_string(&self.locale)
            .to_string();

        ValueRef::Owned(res)
    }
}

// impl_read_state!(DateTime<FixedOffset>, DateStyle, TimeStyle, TimezoneStyle);