use cocoa::foundation::NSInteger;

mod save_dialog;
mod open_dialog;
mod color_dialog;

pub use save_dialog::SavePanel;
pub use open_dialog::OpenPanel;
pub use color_dialog::ColorPanel;

pub(crate) type NSModalResponse = NSInteger;

pub(crate) const NSModalResponseOK: NSModalResponse = 1;
pub(crate) const NSModalResponseCancel: NSModalResponse = 0;