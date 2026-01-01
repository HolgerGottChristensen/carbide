use carbide_core::event::{Key, ModifierKey};

pub enum TextInputKeyCommand {
    MoveLeft,
    MoveRight,
    SelectLeft,
    SelectRight,
    RemoveLeft,
    RemoveRight,
    JumpWordLeft,
    JumpWordRight,
    JumpSelectWordLeft,
    JumpSelectWordRight,
    RemoveWordLeft,
    RemoveWordRight,
    DuplicateLeft,
    DuplicateRight,
    Copy,
    Paste,
    Clip,
    SelectAll,
    RemoveAll,
    JumpToLeft,
    JumpToRight,
    JumpSelectToLeft,
    JumpSelectToRight,
    Enter,
    Undefined,
}

#[cfg(target_os = "windows")]
impl From<(&Key, &ModifierKey)> for TextInputKeyCommand {
    fn from((key, modifier): (&Key, &ModifierKey)) -> Self {
        match (key, modifier) {
            (Key::ArrowLeft, &ModifierKey::EMPTY) => TextInputKeyCommand::MoveLeft,
            (Key::ArrowLeft, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectLeft,
            (Key::ArrowLeft, &ModifierKey::CONTROL) => TextInputKeyCommand::JumpWordLeft,
            (Key::ArrowLeft, &ModifierKey::CTRL_SHIFT) => TextInputKeyCommand::JumpSelectWordLeft,
            (Key::ArrowRight, &ModifierKey::EMPTY) => TextInputKeyCommand::MoveRight,
            (Key::ArrowRight, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectRight,
            (Key::ArrowRight, &ModifierKey::CONTROL) => TextInputKeyCommand::JumpWordRight,
            (Key::ArrowRight, &ModifierKey::CTRL_SHIFT) => TextInputKeyCommand::JumpSelectWordRight,
            (Key::Backspace, &ModifierKey::EMPTY) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::CONTROL) => TextInputKeyCommand::RemoveWordLeft,
            (Key::Delete, &ModifierKey::EMPTY) => TextInputKeyCommand::RemoveRight,
            (Key::Delete, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveAll,
            (Key::Delete, &ModifierKey::CONTROL) => TextInputKeyCommand::RemoveWordRight,
            (Key::Character(c), &ModifierKey::CONTROL) if c == "c" => TextInputKeyCommand::Copy,
            (Key::Character(c), &ModifierKey::CONTROL) if c == "v" => TextInputKeyCommand::Paste,
            (Key::Character(c), &ModifierKey::CONTROL) if c == "x" => TextInputKeyCommand::Clip,
            (Key::Character(c), &ModifierKey::CONTROL) if c == "a" => TextInputKeyCommand::SelectAll,
            (Key::Character(c), &ModifierKey::CONTROL) if c == "d"=> TextInputKeyCommand::DuplicateRight,
            (Key::Character(c), &ModifierKey::CTRL_SHIFT) if c == "d" => TextInputKeyCommand::DuplicateLeft,
            (Key::Home, &ModifierKey::EMPTY) => TextInputKeyCommand::JumpToLeft,
            (Key::Home, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToLeft,
            (Key::End, &ModifierKey::EMPTY) => TextInputKeyCommand::JumpToRight,
            (Key::End, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToRight,
            (Key::Enter, &ModifierKey::EMPTY) => TextInputKeyCommand::Enter,
            _ => TextInputKeyCommand::Undefined,
        }
    }
}

#[cfg(target_os = "macos")]
impl From<(&Key, &ModifierKey)> for TextInputKeyCommand {
    fn from((key, modifier): (&Key, &ModifierKey)) -> Self {
        match (key, modifier) {
            (Key::ArrowLeft, &ModifierKey::EMPTY) => TextInputKeyCommand::MoveLeft,
            (Key::ArrowLeft, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectLeft,
            (Key::ArrowLeft, &ModifierKey::ALT) => TextInputKeyCommand::JumpWordLeft,
            (Key::ArrowLeft, &ModifierKey::SUPER) => TextInputKeyCommand::JumpToLeft,
            (Key::ArrowLeft, &ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordLeft,
            (Key::ArrowLeft, &ModifierKey::SHIFT_SUPER) => TextInputKeyCommand::JumpSelectToLeft,

            (Key::ArrowRight, &ModifierKey::EMPTY) => TextInputKeyCommand::MoveRight,
            (Key::ArrowRight, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectRight,
            (Key::ArrowRight, &ModifierKey::ALT) => TextInputKeyCommand::JumpWordRight,
            (Key::ArrowRight, &ModifierKey::SUPER) => TextInputKeyCommand::JumpToRight,
            (Key::ArrowRight, &ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordRight,
            (Key::ArrowRight, &ModifierKey::SHIFT_SUPER) => TextInputKeyCommand::JumpSelectToRight,

            (Key::Backspace, &ModifierKey::EMPTY) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::ALT) => TextInputKeyCommand::RemoveWordLeft,

            (Key::Delete, &ModifierKey::EMPTY) => TextInputKeyCommand::RemoveRight,
            (Key::Delete, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveAll,
            (Key::Delete, &ModifierKey::ALT) => TextInputKeyCommand::RemoveWordRight,

            (Key::Character(c), &ModifierKey::SUPER) if c == "c" => TextInputKeyCommand::Copy,
            (Key::Character(c), &ModifierKey::SUPER) if c == "v" => TextInputKeyCommand::Paste,
            (Key::Character(c), &ModifierKey::SUPER) if c == "x" => TextInputKeyCommand::Clip,
            (Key::Character(c), &ModifierKey::SUPER) if c == "a" => TextInputKeyCommand::SelectAll,
            (Key::Character(c), &ModifierKey::SUPER) if c == "d" => TextInputKeyCommand::DuplicateRight,
            (Key::Character(c), &ModifierKey::SHIFT_SUPER) if c == "d" => TextInputKeyCommand::DuplicateLeft,

            (Key::Home, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToLeft,
            (Key::End, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToRight,
            (Key::Enter, &ModifierKey::EMPTY) => TextInputKeyCommand::Enter,
            _ => TextInputKeyCommand::Undefined,
        }
    }
}
