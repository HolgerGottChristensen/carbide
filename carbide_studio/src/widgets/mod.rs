mod toolbar;
mod widget_viewer;
mod widget_outline;
mod widget_details;

pub use toolbar::*;
pub use widget_viewer::*;
pub use widget_outline::*;
pub use widget_details::*;

use carbide::state::ReadState;
use carbide::widget::AnyWidget;
