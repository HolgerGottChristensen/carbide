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
            (Key::Left, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::MoveLeft,
            (Key::Left, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectLeft,
            (Key::Left, &ModifierKey::CTRL) => TextInputKeyCommand::JumpWordLeft,
            (Key::Left, &ModifierKey::CTRL_SHIFT) => TextInputKeyCommand::JumpSelectWordLeft,
            (Key::Right, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::MoveRight,
            (Key::Right, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectRight,
            (Key::Right, &ModifierKey::CTRL) => TextInputKeyCommand::JumpWordRight,
            (Key::Right, &ModifierKey::CTRL_SHIFT) => TextInputKeyCommand::JumpSelectWordRight,
            (Key::Backspace, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::CTRL) => TextInputKeyCommand::RemoveWordLeft,
            (Key::Delete, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::RemoveRight,
            (Key::Delete, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveAll,
            (Key::Delete, &ModifierKey::CTRL) => TextInputKeyCommand::RemoveWordRight,
            (Key::C, &ModifierKey::CTRL) => TextInputKeyCommand::Copy,
            (Key::V, &ModifierKey::CTRL) => TextInputKeyCommand::Paste,
            (Key::X, &ModifierKey::CTRL) => TextInputKeyCommand::Clip,
            (Key::A, &ModifierKey::CTRL) => TextInputKeyCommand::SelectAll,
            (Key::D, &ModifierKey::CTRL) => TextInputKeyCommand::DuplicateRight,
            (Key::D, &ModifierKey::CTRL_SHIFT) => TextInputKeyCommand::DuplicateLeft,
            (Key::Home, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::JumpToLeft,
            (Key::Home, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToLeft,
            (Key::End, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::JumpToRight,
            (Key::End, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToRight,
            (Key::Return, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::Enter,
            _ => TextInputKeyCommand::Undefined,
        }
    }
}

#[cfg(target_os = "macos")]
impl From<(&Key, &ModifierKey)> for TextInputKeyCommand {
    fn from((key, modifier): (&Key, &ModifierKey)) -> Self {
        match (key, modifier) {
            (Key::ArrowLeft, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::MoveLeft,
            (Key::ArrowLeft, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectLeft,
            (Key::ArrowLeft, &ModifierKey::ALT) => TextInputKeyCommand::JumpWordLeft,
            (Key::ArrowLeft, &ModifierKey::META) => TextInputKeyCommand::JumpToLeft,
            (Key::ArrowLeft, &ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordLeft,
            (Key::ArrowLeft, &ModifierKey::SHIFT_GUI) => TextInputKeyCommand::JumpSelectToLeft,

            (Key::ArrowRight, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::MoveRight,
            (Key::ArrowRight, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectRight,
            (Key::ArrowRight, &ModifierKey::ALT) => TextInputKeyCommand::JumpWordRight,
            (Key::ArrowRight, &ModifierKey::META) => TextInputKeyCommand::JumpToRight,
            (Key::ArrowRight, &ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordRight,
            (Key::ArrowRight, &ModifierKey::SHIFT_GUI) => TextInputKeyCommand::JumpSelectToRight,

            (Key::Backspace, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::ALT) => TextInputKeyCommand::RemoveWordLeft,

            (Key::Delete, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::RemoveRight,
            (Key::Delete, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveAll,
            (Key::Delete, &ModifierKey::ALT) => TextInputKeyCommand::RemoveWordRight,

            (Key::Character(c), &ModifierKey::META) if c == "c" => TextInputKeyCommand::Copy,
            (Key::Character(c), &ModifierKey::META) if c == "v" => TextInputKeyCommand::Paste,
            (Key::Character(c), &ModifierKey::META) if c == "x" => TextInputKeyCommand::Clip,
            (Key::Character(c), &ModifierKey::META) if c == "a" => TextInputKeyCommand::SelectAll,
            (Key::Character(c), &ModifierKey::META) if c == "d" => TextInputKeyCommand::DuplicateRight,
            (Key::Character(c), &ModifierKey::SHIFT_GUI) if c == "d" => TextInputKeyCommand::DuplicateLeft,

            (Key::Home, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToLeft,
            (Key::End, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToRight,
            (Key::Enter, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::Enter,
            _ => TextInputKeyCommand::Undefined,
        }
    }
}
