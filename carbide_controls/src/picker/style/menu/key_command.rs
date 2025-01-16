use carbide::event::{Key, KeyboardEvent};

pub(super) enum PopupButtonKeyCommand {
    Next,
    Prev,
    Select,
    Close,
    Open,
}

impl PartialEq<PopupButtonKeyCommand> for &KeyboardEvent {
    fn eq(&self, other: &PopupButtonKeyCommand) -> bool {
        match other {
            PopupButtonKeyCommand::Next => {
                matches!(self, KeyboardEvent::Press { key: Key::ArrowDown, .. })
            }
            PopupButtonKeyCommand::Prev => {
                matches!(self, KeyboardEvent::Press { key: Key::ArrowUp, .. })
            }
            PopupButtonKeyCommand::Select => {
                matches!(self, KeyboardEvent::Press { key: Key::Enter, .. })
            }
            PopupButtonKeyCommand::Close => {
                matches!(self, KeyboardEvent::Press { key: Key::Escape, .. })
            }
            PopupButtonKeyCommand::Open => {
                matches!(self, KeyboardEvent::Press { key: Key::Space, .. } | KeyboardEvent::Press { key: Key::Enter, .. })
            }
        }
    }
}