pub use button::*;
use carbide_core::environment::Environment;
use carbide_core::state::{EnvMap1, Map1};
pub use check_box::*;
pub use list::*;
pub use plain::*;
pub use pop_up_button::*;
pub use radio_button::*;
pub use slider::*;
pub use switch::*;
pub use text_input::*;
pub use types::CheckBoxValue;
pub use controls_ext::*;
pub use help::*;
pub use labelled::*;
pub use calendar::*;
use carbide::focus::{Focus, Refocus};
use carbide::state::{ReadState, State};
use carbide::widget::{MouseAreaAction, MouseAreaActionContext};
pub use date_picker::*;
pub use toggle_style::{SwitchStyle, CheckboxStyle};

extern crate carbide_core as carbide;

#[macro_export]
macro_rules! capture {
    ($([$($t:ident),*],)? $({$($u:ident),*},)? |$($a:ident: $typ:ty),*| $b:block) => {
        {
            $($(let $t = $t.clone();)*)?
            $($(let $u = $u.clone();)*)?
            #[allow(unused_variables)]
            #[allow(unused_imports)]
            #[allow(unused_mut)]
            move |$($a: $typ),*| {

                use carbide_core::state::State;
                $($(let mut $t = $t.clone();)*)?
                $($(let mut $u = $u.clone();)*)?
                //{
                    //$($(let mut $t = $t.value_mut();)*)?
                    $b
                //}
                //$($($t.update_dependent();)*)?
            }
        }
    };
}



mod button;
mod check_box;
mod list;
mod plain;
mod pop_up_button;
mod radio_button;
mod slider;
mod switch;
mod text_input;
mod types;
mod controls_ext;
mod help;
mod labelled;
mod calendar;
mod date_picker;
mod toggle_style;

type EnabledState = EnvMap1<fn(&Environment, &i32) -> bool, i32, bool, i32>;

pub fn enabled_state() -> EnabledState {
    Map1::read_map_env(0, |env, _| {
        // Look up enabled in the environment, or default to true of nothing is specified
        env.bool("enabled").unwrap_or(true)
    })
}

#[derive(Debug, Clone)]
pub(crate) struct UnfocusAction<F>(F) where F: State<T=Focus>;

impl<F: State<T=Focus>> MouseAreaAction for UnfocusAction<F> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.0.sync(ctx.env);
        if *self.0.value() == Focus::Focused {
            self.0.set_value(Focus::FocusReleased);
            ctx.env.request_focus(Refocus::FocusRequest);
        }
    }
}
