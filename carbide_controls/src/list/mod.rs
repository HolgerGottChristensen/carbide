mod list;
mod list_selection;
mod selectable_delegate;
mod style;
mod row_delegate;
mod row_styled;

use carbide::event::ModifierKey;
pub use list::*;
pub use style::*;
pub(crate) use list_selection::*;
pub(crate) use selectable_delegate::*;


pub(crate) const MULTI_SELECTION_MODIFIER: ModifierKey = if cfg!(target_os = "macos") {
    ModifierKey::SUPER
} else {
    ModifierKey::CONTROL
};
pub(crate) const LIST_SELECTION_MODIFIER: ModifierKey = ModifierKey::SHIFT;

pub(crate) const LIST_SELECTION_AND_MULTI_SELECTION_MODIFIER: ModifierKey = if cfg!(target_os = "macos") {
    ModifierKey::SHIFT_SUPER
} else {
    ModifierKey::CTRL_SHIFT
};