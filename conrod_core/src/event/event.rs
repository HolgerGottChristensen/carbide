use event::input::Input;
use event::ui::UiEvent;

impl From<Input> for Event {
    fn from(input: Input) -> Self {
        Event::Raw(input)
    }
}

impl From<UiEvent> for Event {
    fn from(ui: UiEvent) -> Self {
        Event::Ui(ui)
    }
}

/// Enum containing all the events that the `Ui` may provide.
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    /// Represents a raw `input::Input` event.
    Raw(Input),
    /// Events that have been interpreted from `backend::RawEvent`s by the `Ui`.
    ///
    /// Most events usually
    Ui(UiEvent)
}

