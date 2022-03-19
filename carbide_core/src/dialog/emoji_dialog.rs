#[cfg(target_os = "macos")]
use crate::platform::mac::open_emoji_dialog;

pub struct EmojiDialog;

impl EmojiDialog {
    pub fn new() -> Self {
        EmojiDialog
    }

    #[cfg(target_os = "macos")]
    pub fn open(self) {
        open_emoji_dialog()
    }

    #[cfg(not(target_os = "macos"))]
    pub fn open(mut self) {
        todo!()
    }
}