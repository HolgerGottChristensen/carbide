use event::input::Input;
use event::ui::UiEvent;

impl From<Input> for OldEvent {
    fn from(input: Input) -> Self {
        OldEvent::Raw(input)
    }
}

impl From<UiEvent> for OldEvent {
    fn from(ui: UiEvent) -> Self {
        OldEvent::Ui(ui)
    }
}

/// Enum containing all the events that the `Ui` may provide.
#[derive(Clone, PartialEq, Debug)]
pub enum OldEvent {
    /// Represents a raw `input::Input` event.
    Raw(Input),
    /// Events that have been interpreted from `backend::RawEvent`s by the `Ui`.
    ///
    /// Most events usually
    Ui(UiEvent)
}

