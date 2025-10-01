pub use list::*;
pub use plain::*;
pub use slider::*;
pub use text_input::*;
pub use controls_ext::*;
pub use help::*;
pub use calendar::*;
use carbide::environment::EnvironmentKey;
use carbide::focus::{Focus, FocusManager, Refocus};
use carbide::state::{KeyState, ReadState, State};
use carbide::widget::{MouseAreaAction, MouseAreaActionContext, OverlayManager, Widget};
pub use date_picker::*;

extern crate carbide_core as carbide;

#[doc(hidden)]
pub mod __private {
    pub mod core {
        pub use carbide_core::*;
    }
}

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

                use $crate::__private::core::state::State;
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



mod list;
mod plain;
mod text_input;
mod controls_ext;
mod help;
mod calendar;
mod date_picker;
pub mod toggle;
pub mod picker;
pub mod identifiable;
mod selectable;
mod labelled;
pub mod button;
mod slider;
pub mod context_menu;

pub type EnabledState = KeyState<EnabledKey>;

#[derive(Debug, Copy, Clone)]
pub struct EnabledKey;
impl EnvironmentKey for EnabledKey {
    type Value = bool;
}

#[derive(Debug, Clone)]
pub(crate) struct UnfocusAction<F>(F) where F: State<T=Focus>;

impl<F: State<T=Focus>> MouseAreaAction for UnfocusAction<F> {
    fn call(&mut self, ctx: MouseAreaActionContext) {
        self.0.sync(ctx.env);
        if *self.0.value() == Focus::Focused {
            self.0.set_value(Focus::FocusReleased);
            FocusManager::get(ctx.env, |manager| {
                manager.request_focus(Refocus::FocusRequest)
            });
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ControlsOverlayKey;

impl EnvironmentKey for ControlsOverlayKey {
    type Value = OverlayManager;
}

pub fn controls_overlay<C: Widget>(c: C) -> impl Widget {
    c.overlay::<ControlsOverlayKey>().steal_events()
}