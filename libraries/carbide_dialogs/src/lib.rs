extern crate carbide_core as carbide;
pub mod open_dialog;
mod dialogs_ext;
mod file_type;
pub mod save_dialog;
pub mod color_dialog;

pub use dialogs_ext::*;
pub use file_type::*;

#[derive(Copy, Clone, Debug)]
pub struct NativeStyle;