use raw_window_handle::RawWindowHandle;

use carbide_core::environment::{EnvironmentKey, Shared};

/// The raw handle of the window currently being processed. Set by the windowing backend while
/// it dispatches events to and renders a window, so native widgets such as file dialogs can
/// attach to it.
#[derive(Copy, Clone, Debug)]
pub struct WindowHandleKey;

impl EnvironmentKey for WindowHandleKey {
    type Value = RawWindowHandle;
    type Access = Shared;
}
