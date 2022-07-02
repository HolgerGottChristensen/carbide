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
            (Key::Left, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::MoveLeft,
            (Key::Left, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectLeft,
            (Key::Left, &ModifierKey::ALT) => TextInputKeyCommand::JumpWordLeft,
            (Key::Left, &ModifierKey::GUI) => TextInputKeyCommand::JumpToLeft,
            (Key::Left, &ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordLeft,
            (Key::Left, &ModifierKey::SHIFT_GUI) => TextInputKeyCommand::JumpSelectToLeft,

            (Key::Right, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::MoveRight,
            (Key::Right, &ModifierKey::SHIFT) => TextInputKeyCommand::SelectRight,
            (Key::Right, &ModifierKey::ALT) => TextInputKeyCommand::JumpWordRight,
            (Key::Right, &ModifierKey::GUI) => TextInputKeyCommand::JumpToRight,
            (Key::Right, &ModifierKey::SHIFT_ALT) => TextInputKeyCommand::JumpSelectWordRight,
            (Key::Right, &ModifierKey::SHIFT_GUI) => TextInputKeyCommand::JumpSelectToRight,

            (Key::Backspace, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveLeft,
            (Key::Backspace, &ModifierKey::ALT) => TextInputKeyCommand::RemoveWordLeft,

            (Key::Delete, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::RemoveRight,
            (Key::Delete, &ModifierKey::SHIFT) => TextInputKeyCommand::RemoveAll,
            (Key::Delete, &ModifierKey::ALT) => TextInputKeyCommand::RemoveWordRight,

            (Key::C, &ModifierKey::GUI) => TextInputKeyCommand::Copy,
            (Key::V, &ModifierKey::GUI) => TextInputKeyCommand::Paste,
            (Key::X, &ModifierKey::GUI) => TextInputKeyCommand::Clip,
            (Key::A, &ModifierKey::GUI) => TextInputKeyCommand::SelectAll,
            (Key::D, &ModifierKey::GUI) => TextInputKeyCommand::DuplicateRight,
            (Key::D, &ModifierKey::SHIFT_GUI) => TextInputKeyCommand::DuplicateLeft,

            (Key::Home, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToLeft,
            (Key::End, &ModifierKey::SHIFT) => TextInputKeyCommand::JumpSelectToRight,
            (Key::Return, &ModifierKey::NO_MODIFIER) => TextInputKeyCommand::Enter,
            _ => TextInputKeyCommand::Undefined,
        }
    }
}
