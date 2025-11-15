use crate::args::{Arg, Args, LocalizedArg};
use crate::locale_ext::LocaleKey;
use crate::localizable::Localizable;
use crate::{locale, LANGUAGES};
use carbide_core::environment::Environment;
use carbide_core::state::{AnyReadState, ReadState, StateSync, ValueRef};
use fluent_for_carbide::types::{FluentDateTime, FluentNumber};
use fluent_for_carbide::FluentArgs;
use icu::locid::Locale;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct LocalizedString<K, S, V> where K: Localizable, V: Args, S: ReadState<T=K> {
    key: S,
    args: V,
    locale: Locale,
}

impl<K: Localizable, S: ReadState<T=K>> LocalizedString<K, S, ()> {
    pub fn new(key: S) -> LocalizedString<K, S, ()> {
        LocalizedString {
            key,
            args: (),
            locale: locale!("en"),
        }
    }
}

impl<K: Localizable, S: ReadState<T=K>, V: Args> LocalizedString<K, S, V> {
    pub fn arg<G: Arg, T: ReadState<T=G>>(self, key: &'static str, arg: T) -> LocalizedString<K, S, impl Args> {
        LocalizedString {
            key: self.key,
            args: self.args.push(key, arg),
            locale: self.locale,
        }
    }
}


impl<K: Localizable, S: ReadState<T=K>, V: Args> StateSync for LocalizedString<K, S, V> {
    fn sync(&mut self, env: &mut Environment) -> bool {
        if let Some(locale) = env.get::<LocaleKey>() {
            //println!("Synced locale: {}", locale);
            self.locale = locale.clone();
            true
        } else {
            false
        }
    }
}

impl<K: Localizable, S: ReadState<T=K>, V: Args> AnyReadState for LocalizedString<K, S, V> {
    type T = String;

    fn value_dyn(&self) -> ValueRef<Self::T> {
        let languages = &LANGUAGES;

        let res = if let Some(bundle) = languages.get(&self.locale) {
            let binding = self.key.value();
            let mut split_value = binding.get().split('.');

            bundle.get_message(split_value.next().unwrap()).and_then(|message| {
                if let Some(attribute) = split_value.next() {
                    message.attributes().find(|a| a.id() == attribute).map(|a| a.value())
                } else {
                    message.value()
                }
            }).map(|pattern| {
                let mut args = FluentArgs::new();

                for (key, value) in self.args.iter() {
                    match value {
                        LocalizedArg::String(s) => {
                            args.set(key, s);
                        }
                        LocalizedArg::Number(n, options) => {
                            args.set(key, FluentNumber {
                                value: n,
                                options,
                            });
                        }
                        LocalizedArg::Date(date, options) => {
                            args.set(key, FluentDateTime {
                                value: date,
                                options,
                            });
                        }
                        LocalizedArg::Str(s) => {
                            args.set(key, s);
                        }
                    }
                }

                bundle.format_pattern(pattern, Some(&args), &mut vec![]).to_string()
            }).unwrap_or_else(|| {
                self.key.value().get().to_string()
            })
        } else {
            self.key.value().get().to_string()
        };




        ValueRef::Owned(res)
    }
}