use std::fmt::Debug;
use chrono::{DateTime, FixedOffset};
use fluent::FluentArgs;
use fluent::types::{FluentDateTime, FluentDateTimeOptions, FluentNumber, FluentNumberOptions};
use icu::locid::Locale;
use carbide_core::environment::Environment;
use carbide_core::impl_read_state;
use carbide_core::state::{AnyReadState, IntoReadState, NewStateSync, ReadState, ValueRef, ValueState};
use crate::{LANGUAGES, locale};
use crate::args::{Arg, Args, LocalizedArg};
use crate::locale_ext::LOCALE_IDENT;
use crate::localizable::Localizable;

pub type Number = fluent::types::FluentNumber;
pub type NumberStyle = fluent::types::FluentNumberStyle;
pub type NumberNotation = fluent::types::FluentNumberNotation;
pub type CurrencyDisplayStyle = fluent::types::FluentNumberCurrencyDisplayStyle;
pub type NumberGrouping = fluent::types::FluentNumberGrouping;
pub type RoundingMode = fluent::types::FluentNumberRoundingMode;

#[derive(Debug, Clone)]
pub struct LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11>
    where
        V: Into<Number>,
        S: ReadState<T=V>,
        S1: ReadState<T=NumberStyle>,
        S2: ReadState<T=NumberNotation>,
        S3: ReadState<T=Option<String>>,
        S4: ReadState<T=CurrencyDisplayStyle>,
        S5: ReadState<T=NumberGrouping>,
        S6: ReadState<T=Option<usize>>,
        S7: ReadState<T=Option<usize>>,
        S8: ReadState<T=Option<usize>>,
        S9: ReadState<T=Option<usize>>,
        S10: ReadState<T=Option<usize>>,
        S11: ReadState<T=RoundingMode>,
{
    value: S,
    style: S1,
    notation: S2,
    currency: S3,
    currency_display: S4,
    use_grouping: S5,
    minimum_integer_digits: S6,
    minimum_fraction_digits: S7,
    maximum_fraction_digits: S8,
    minimum_significant_digits: S9,
    maximum_significant_digits: S10,
    rounding_mode: S11,
    locale: Locale,
}

impl LocalizedNumber<
    u32,
    u32,
    ValueState<NumberStyle>,
    ValueState<NumberNotation>,
    ValueState<Option<String>>,
    ValueState<CurrencyDisplayStyle>,
    ValueState<NumberGrouping>,
    Option<usize>,
    Option<usize>,
    Option<usize>,
    Option<usize>,
    Option<usize>,
    ValueState<RoundingMode>,
> {
    pub fn new<V: Into<Number>, S: ReadState<T=V>>(value: S) -> LocalizedNumber<
        V,
        S,
        ValueState<NumberStyle>,
        ValueState<NumberNotation>,
        ValueState<Option<String>>,
        ValueState<CurrencyDisplayStyle>,
        ValueState<NumberGrouping>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
        ValueState<RoundingMode>
    > {
        LocalizedNumber {
            value,
            style: ValueState::new(NumberStyle::default()),
            notation: ValueState::new(NumberNotation::default()),
            currency: ValueState::new(None),
            currency_display: ValueState::new(CurrencyDisplayStyle::default()),
            use_grouping: ValueState::new(NumberGrouping::default()),
            minimum_integer_digits: None,
            minimum_fraction_digits: None,
            maximum_fraction_digits: None,
            minimum_significant_digits: None,
            maximum_significant_digits: None,
            rounding_mode: ValueState::new(RoundingMode::default()),
            locale: locale!("en"),
        }
    }
}

impl<
    V: Into<Number>,
    S: ReadState<T=V>,
    S1: ReadState<T=NumberStyle>,
    S2: ReadState<T=NumberNotation>,
    S3: ReadState<T=Option<String>>,
    S4: ReadState<T=CurrencyDisplayStyle>,
    S5: ReadState<T=NumberGrouping>,
    S6: ReadState<T=Option<usize>>,
    S7: ReadState<T=Option<usize>>,
    S8: ReadState<T=Option<usize>>,
    S9: ReadState<T=Option<usize>>,
    S10: ReadState<T=Option<usize>>,
    S11: ReadState<T=RoundingMode>,
> LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11> {
    pub fn style<N: IntoReadState<NumberStyle>>(self, style: N) -> LocalizedNumber<V, S, N::Output, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: style.into_read_state(),
            notation: self.notation,
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn notation<N: IntoReadState<NumberNotation>>(self, notation: N) -> LocalizedNumber<V, S, S1, N::Output, S3, S4, S5, S6, S7, S8, S9, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: notation.into_read_state(),
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn currency<N: IntoReadState<Option<String>>>(self, currency: N) -> LocalizedNumber<V, S, S1, S2, N::Output, S4, S5, S6, S7, S8, S9, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: currency.into_read_state(),
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn currency_display<N: IntoReadState<CurrencyDisplayStyle>>(self, currency_display: N) -> LocalizedNumber<V, S, S1, S2, S3, N::Output, S5, S6, S7, S8, S9, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: self.currency,
            currency_display: currency_display.into_read_state(),
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn use_grouping<N: IntoReadState<NumberGrouping>>(self, use_grouping: N) -> LocalizedNumber<V, S, S1, S2, S3, S4, N::Output, S6, S7, S8, S9, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: use_grouping.into_read_state(),
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn minimum_integer_digits<N: IntoReadState<Option<usize>>>(self, minimum_integer_digits: N) -> LocalizedNumber<V, S, S1, S2, S3, S4, S5, N::Output, S7, S8, S9, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: minimum_integer_digits.into_read_state(),
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn minimum_fraction_digits<N: IntoReadState<Option<usize>>>(self, minimum_fraction_digits: N) -> LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, N::Output, S8, S9, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: minimum_fraction_digits.into_read_state(),
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn maximum_fraction_digits<N: IntoReadState<Option<usize>>>(self, maximum_fraction_digits: N) -> LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, S7, N::Output, S9, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: maximum_fraction_digits.into_read_state(),
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn minimum_significant_digits<N: IntoReadState<Option<usize>>>(self, minimum_significant_digits: N) -> LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, S7, S8, N::Output, S10, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: minimum_significant_digits.into_read_state(),
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn maximum_significant_digits<N: IntoReadState<Option<usize>>>(self, maximum_significant_digits: N) -> LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, S7, S8, S9, N::Output, S11> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: maximum_significant_digits.into_read_state(),
            rounding_mode: self.rounding_mode,
            locale: self.locale,
        }
    }

    pub fn rounding_mode<N: IntoReadState<RoundingMode>>(self, rounding_mode: N) -> LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, N::Output> {
        LocalizedNumber {
            value: self.value,
            style: self.style,
            notation: self.notation,
            currency: self.currency,
            currency_display: self.currency_display,
            use_grouping: self.use_grouping,
            minimum_integer_digits: self.minimum_integer_digits,
            minimum_fraction_digits: self.minimum_fraction_digits,
            maximum_fraction_digits: self.maximum_fraction_digits,
            minimum_significant_digits: self.minimum_significant_digits,
            maximum_significant_digits: self.maximum_significant_digits,
            rounding_mode: rounding_mode.into_read_state(),
            locale: self.locale,
        }
    }

}

impl<
    V: Into<Number>,
    S: ReadState<T=V>,
    S1: ReadState<T=NumberStyle>,
    S2: ReadState<T=NumberNotation>,
    S3: ReadState<T=Option<String>>,
    S4: ReadState<T=CurrencyDisplayStyle>,
    S5: ReadState<T=NumberGrouping>,
    S6: ReadState<T=Option<usize>>,
    S7: ReadState<T=Option<usize>>,
    S8: ReadState<T=Option<usize>>,
    S9: ReadState<T=Option<usize>>,
    S10: ReadState<T=Option<usize>>,
    S11: ReadState<T=RoundingMode>,
> NewStateSync for LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        self.value.sync(env);
        self.style.sync(env);
        self.notation.sync(env);
        self.currency.sync(env);
        self.currency_display.sync(env);
        self.use_grouping.sync(env);
        self.minimum_integer_digits.sync(env);
        self.minimum_fraction_digits.sync(env);
        self.maximum_fraction_digits.sync(env);
        self.minimum_significant_digits.sync(env);
        self.maximum_significant_digits.sync(env);
        self.rounding_mode.sync(env);

        if let Some(locale) = env.value::<&'static str, Locale>(LOCALE_IDENT) {
            self.locale = locale.clone();
            true
        } else {
            false
        }
    }
}

impl<
    V: Into<Number> + Debug + Clone + 'static,
    S: ReadState<T=V>,
    S1: ReadState<T=NumberStyle>,
    S2: ReadState<T=NumberNotation>,
    S3: ReadState<T=Option<String>>,
    S4: ReadState<T=CurrencyDisplayStyle>,
    S5: ReadState<T=NumberGrouping>,
    S6: ReadState<T=Option<usize>>,
    S7: ReadState<T=Option<usize>>,
    S8: ReadState<T=Option<usize>>,
    S9: ReadState<T=Option<usize>>,
    S10: ReadState<T=Option<usize>>,
    S11: ReadState<T=RoundingMode>,
> AnyReadState for LocalizedNumber<V, S, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11> {
    type T = String;

    fn value_dyn(&self) -> ValueRef<Self::T> {
        let mut value = self.value.value().clone().into();
        let style = self.style.value();
        let notation = self.notation.value();
        let currency = self.currency.value();
        let currency_display = self.currency_display.value();
        let use_grouping = self.use_grouping.value();
        let minimum_integer_digits = self.minimum_integer_digits.value();
        let minimum_fraction_digits = self.minimum_fraction_digits.value();
        let maximum_fraction_digits = self.maximum_fraction_digits.value();
        let minimum_significant_digits = self.minimum_significant_digits.value();
        let maximum_significant_digits = self.maximum_significant_digits.value();
        let rounding_mode = self.rounding_mode.value();

        value.options = FluentNumberOptions {
            style: *style,
            notation: *notation,
            currency: currency.clone(),
            currency_display: *currency_display,
            use_grouping: *use_grouping,
            minimum_integer_digits: *minimum_integer_digits,
            minimum_fraction_digits: *minimum_fraction_digits,
            maximum_fraction_digits: *maximum_fraction_digits,
            minimum_significant_digits: *minimum_significant_digits,
            maximum_significant_digits: *maximum_significant_digits,
            rounding_mode: *rounding_mode,
        };

        ValueRef::Owned(value.as_string(&self.locale).to_string())
    }
}

// impl_read_state!(DateTime<FixedOffset>, DateStyle, TimeStyle, TimezoneStyle);