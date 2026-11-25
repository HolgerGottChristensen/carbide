use carbide::focus::Focus;
use carbide::state::{ReadState, State};
use carbide::widget::{Action, AnyWidget, Widget, WidgetProperties};
use carbide::widget::properties::WidgetKindSimple;
use crate::button::Button;

pub trait AnyMenuWidget: AnyWidget {

}

impl<A: Action + Clone + 'static, L: Widget + WidgetProperties<Kind=WidgetKindSimple>> AnyMenuWidget for Button<A, L> {

}