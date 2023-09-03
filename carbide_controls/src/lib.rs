//pub use button::Button;
pub use check_box::*;
pub use list::*;
pub use navigation_stack::NavigationStack;
pub use plain::*;
//pub use pop_up_button::PopUpButton;
//pub use radio_button::RadioButton;
//pub use slider::Slider;
pub use switch::Switch;
//pub use text_input::TextInput;
//pub use types::CheckBoxState;
pub use types::CheckBoxValue;

#[macro_export]
macro_rules! capture {
    ($([$($t:ident),*],)? $({$($u:ident),*},)? |$($a:ident: $typ:ty),*| $b:block) => {
        {
            $($(let $t = $t.clone();)*)?
            $($(let $u = $u.clone();)*)?
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
mod text_input;
mod types;
