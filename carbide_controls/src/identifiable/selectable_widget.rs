use carbide::state::{AnyReadState};
use carbide::widget::{AnyWidget, WidgetExt};
use std::fmt::Debug;

pub trait AnySelectableWidget: AnyWidget {
    fn as_widget(&self) -> &dyn AnyWidget;
    fn selected(&self) -> &dyn AnyReadState<T=bool>;
}

dyn_clone::clone_trait_object!(AnySelectableWidget);

pub trait SelectableWidget: AnySelectableWidget + WidgetExt + Clone {}

impl<W> SelectableWidget for W where W: AnySelectableWidget + WidgetExt + Clone {}