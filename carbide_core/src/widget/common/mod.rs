pub use common_widget::*;
pub use into_widget::*;
pub use widget::{AnyWidget, Widget};
pub use widget_ext::*;
pub use widget_sequence::*;
pub use widget_sync::WidgetSync;
pub use widget_style::WidgetStyle;
pub use content::*;
pub use widget_properties::WidgetProperties;

pub mod properties {
    pub use super::widget_properties::{
        WidgetKindIgnore,
        WidgetKindSimple,
        WidgetKindProxy,
        WidgetKindDynamic,
        Kind,
        WidgetKind
    };
}

mod common_widget;
mod widget;
mod widget_ext;
mod widget_sequence;
mod into_widget;
mod widget_sync;
mod content;
mod widget_properties;
mod widget_style;
