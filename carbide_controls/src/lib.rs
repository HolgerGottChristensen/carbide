#[macro_use]
extern crate carbide_core;
#[macro_use]
extern crate carbide_derive;
extern crate copypasta;
extern crate unicode_segmentation;

pub use button::Button;
pub use check_box::*;
pub use list::List;
pub use plain::*;
pub use pop_up_button::PopUpButton;
pub use radio_button::RadioButton;
pub use switch::Switch;
pub use text_input::TextInput;
pub use types::CheckBoxState;
pub use types::CheckBoxValue;

#[macro_export]
macro_rules! capture {
    ([$($t:ident),*], |$($a:ident: $typ:ty),*| $b:block) => {
        {
            $(let $t = $t.clone();)*
            move |$($a: $typ),*| {
                $(let mut $t = $t.clone();)*
                {
                    $(let mut $t = $t.value_mut();)*
                    $b
                }
                $($t.update_dependent();)*
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
mod switch;
mod text_input;
mod types;
