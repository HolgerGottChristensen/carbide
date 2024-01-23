use icu::locid::{Locale, locale};
use carbide_core::environment::{Environment, EnvironmentStateContainer};
use carbide_core::state::{EnvMap1, IntoReadState, Map1, ReadState};
use carbide_core::widget::{EnvUpdating, WidgetExt};
use carbide_core::state::ReadStateExtNew;

type Localed<C, T, S> = EnvUpdating<C, T, S>;
type LocaleState = EnvMap1<fn(&Environment, &i32) -> Locale, i32, Locale, i32>;

pub const LOCALE_IDENT: &'static str = "current_locale";

pub trait LocaleExt: WidgetExt {
    fn locale<L: IntoReadState<Locale>>(self, locale: L) -> Localed<Self, Locale, L::Output> {
        EnvUpdating::new(LOCALE_IDENT, locale.into_read_state(), self)
    }
}

impl<T> LocaleExt for T where T: WidgetExt {}


pub fn locale_state() -> LocaleState {
    Map1::read_map_env(0, |env, _| {
        // Look up enabled in the environment, or default to true of nothing is specified
        env.value::<&'static str, Locale>(LOCALE_IDENT).cloned().unwrap_or(locale!("en"))
    })
}