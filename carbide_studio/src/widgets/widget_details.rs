use carbide::CommonWidgetImpl;
use carbide::draw::{Dimension, Position};
use carbide::state::{LocalState, Map1};
use carbide::widget::{AnyWidget, CommonWidget, Text, Widget, WidgetExt, WidgetId};
use crate::core::any_studio_widget::AnyStudioWidget;

#[derive(Clone, Debug, Widget)]
pub struct WidgetDetails {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    child: LocalState<Box<dyn AnyWidget>>
}

impl WidgetDetails {
    pub fn new(state: LocalState<Box<dyn AnyWidget>>) -> WidgetDetails {
        WidgetDetails {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            child: state.clone(),
        }
    }
}

impl CommonWidget for WidgetDetails {
    CommonWidgetImpl!(self, position: self.position, dimension: self.dimension, child: self.child);
}