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
                matches!(self, KeyboardEvent::Press(Key::ArrowDown, _))
            }
            PopupButtonKeyCommand::Prev => {
                matches!(self, KeyboardEvent::Press(Key::ArrowUp, _))
            }
            PopupButtonKeyCommand::Select => {
                matches!(self, KeyboardEvent::Press(Key::Enter, _))
            }
            PopupButtonKeyCommand::Close => {
                matches!(self, KeyboardEvent::Press(Key::Escape, _))
            }
            PopupButtonKeyCommand::Open => {
                matches!(self, KeyboardEvent::Press(Key::Space, _) | KeyboardEvent::Press(Key::Enter, _))
            }
        }
    }
}