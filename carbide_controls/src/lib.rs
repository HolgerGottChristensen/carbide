pub use button::*;
use carbide_core::environment::Environment;
use carbide_core::state::{EnvMap1, Map1};
pub use check_box::*;
pub use list::*;
pub use navigation_stack::NavigationStack;
pub use plain::*;
pub use pop_up_button::*;
pub use radio_button::*;
pub use slider::*;
pub use switch::*;
//pub use text_input::*;
pub use types::CheckBoxValue;
pub use controls_ext::*;
pub use help::*;
pub use labelled::*;

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
            move |$($a: $typ),*, modifier: carbide_core::event::ModifierKey| {

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
mod navigation_stack;
mod plain;
mod pop_up_button;
mod radio_button;
mod slider;
mod switch;
//mod text_input;
mod types;
mod controls_ext;
mod help;
mod labelled;


type EnabledState = EnvMap1<fn(&Environment, &i32) -> bool, i32, bool, i32>;

pub fn enabled_state() -> EnabledState {
    Map1::read_map_env(0, |env, _| {
        // Look up enabled in the environment, or default to true of nothing is specified
        env.bool("enabled").unwrap_or(true)
    })
}
