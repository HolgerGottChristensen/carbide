use icu::locid::{Locale, locale};
use carbide_core::environment::{EnvironmentStack, Key};
use carbide_core::state::{EnvMap1, IntoReadState, Map1, ReadState};
use carbide_core::widget::{EnvUpdatingNew2, WidgetExt};

type WithLocale<C, K, V> = EnvUpdatingNew2<C, K, V>;
type LocaleState = EnvMap1<fn(&mut EnvironmentStack, &i32) -> Locale, i32, Locale, i32>;

#[derive(Debug, Copy, Clone)]
pub(crate) struct LocaleKey;
impl Key for LocaleKey {
    type Value = Locale;
}


pub trait LocaleExt: WidgetExt {
    fn locale<L: IntoReadState<Locale>>(self, locale: L) -> WithLocale<Self, impl Key<Value=Locale>, impl ReadState<T=Locale>> {
        EnvUpdatingNew2::<Self, LocaleKey, L::Output>::new(locale.into_read_state(), self)
    }
}

impl<T> LocaleExt for T where T: WidgetExt {}


pub fn locale_state() -> LocaleState {
    Map1::read_map_env(0, |env, _| {
        // Look up enabled in the environment, or default to true of nothing is specified
        env.get::<LocaleKey>().cloned().unwrap_or_else(|| locale!("en"))
    })
}