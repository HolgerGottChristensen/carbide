pub use button::*;
use carbide_core::state::{EnvMap1, Map1};
pub use list::*;
pub use plain::*;
pub use pop_up_button::*;
pub use slider::*;
pub use text_input::*;
pub use types::CheckBoxValue;
pub use controls_ext::*;
pub use help::*;
pub use labelled::*;
pub use calendar::*;
use carbide::environment::{EnvironmentStack, Key};
use carbide::focus::{Focus, FocusManager, Refocus};
use carbide::state::{ReadState, State};
use carbide::widget::{MouseAreaAction, MouseAreaActionContext, OverlayManager, Widget};
pub use date_picker::*;

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
mod list;
mod plain;
mod pop_up_button;
mod slider;
mod text_input;
mod types;
mod controls_ext;
mod help;
mod labelled;
mod calendar;
mod date_picker;
pub mod toggle;
pub mod picker;
pub mod identifiable;

type EnabledState = EnvMap1<fn(&mut EnvironmentStack, &i32) -> bool, i32, bool, i32>;

#[derive(Debug, Copy, Clone)]
pub(crate) struct EnabledKey;
impl Key for EnabledKey {
    type Value = bool;
}

pub fn enabled_state() -> EnabledState {
    Map1::read_map_env(0, |env, _| {
        // Look up enabled in the environment, or default to true of nothing is specified
        let val = env.get::<EnabledKey>().cloned().unwrap_or(true);
        val
    })
}

#[derive(Debug, Clone)]
pub(crate) struct UnfocusAction<F>(F) where F: State<T=Focus>;

impl<F: State<T=Focus>> MouseAreaAction for UnfocusAction<F> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.0.sync(ctx.env_stack);
        if *self.0.value() == Focus::Focused {
            self.0.set_value(Focus::FocusReleased);
            FocusManager::get(ctx.env_stack, |manager| {
                manager.request_focus(Refocus::FocusRequest)
            });
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ControlsOverlayKey;

impl Key for ControlsOverlayKey {
    type Value = OverlayManager;
}

pub fn controls_overlay<C: Widget>(c: C) -> impl Widget {
    c.overlay::<ControlsOverlayKey>().steal_events()
}