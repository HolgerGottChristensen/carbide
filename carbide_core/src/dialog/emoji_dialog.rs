use crate::Color;
use crate::color::WHITE;
use crate::environment::Environment;
use crate::platform::mac::color_dialog::open_color_dialog;
use crate::platform::mac::open_emoji_dialog;

pub struct EmojiDialog;

impl EmojiDialog {
    pub fn new() -> Self {
        EmojiDialog
    }

    pub fn open(mut self) {
        open_emoji_dialog()
    }
}