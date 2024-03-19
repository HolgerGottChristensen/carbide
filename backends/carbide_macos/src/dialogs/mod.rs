use cocoa::foundation::NSInteger;
use objc::msg_send;
use objc::sel;
use objc::class;
use objc::sel_impl;
use cocoa::base::{id, nil};

mod save_dialog;
mod open_dialog;
mod color_dialog;

pub use save_dialog::SavePanel;
pub use open_dialog::OpenPanel;
pub use color_dialog::ColorPanel;

pub(crate) type NSModalResponse = NSInteger;

#[allow(non_upper_case_globals)]
pub(crate) const NSModalResponseOK: NSModalResponse = 1;

#[allow(non_upper_case_globals)]
pub(crate) const NSModalResponseCancel: NSModalResponse = 0;

pub fn open_emoji_dialog() {
    unsafe {
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let () = msg_send![app, orderFrontCharacterPalette: nil];
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileSpecification {
    pub(crate) name: &'static str,
    pub(crate) extensions: &'static [&'static str],
}

impl FileSpecification {
    pub const fn new(name: &'static str, extensions: &'static [&'static str]) -> Self {
        FileSpecification { name, extensions }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn extensions(&self) -> &[&str] {
        self.extensions
    }
}