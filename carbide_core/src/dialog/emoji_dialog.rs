use crate::Color;
use crate::color::WHITE;
use crate::environment::Environment;
#[cfg(target_os = "macos")]
use crate::platform::mac::color_dialog::open_color_dialog;
#[cfg(target_os = "macos")]
use crate::platform::mac::open_emoji_dialog;

pub struct EmojiDialog;

impl EmojiDialog {
    pub fn new() -> Self {
        EmojiDialog
    }

    #[cfg(target_os = "macos")]
    pub fn open(mut self) {
        open_emoji_dialog()
    }

    #[cfg(not(target_os = "macos"))]
    pub fn open(mut self) {
        todo!()
    }
}