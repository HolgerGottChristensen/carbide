use carbide::CommonWidgetImpl;
use carbide::controls::List;
use carbide::draw::{Dimension, Position};
use carbide::state::{LocalState, Map1};
use carbide::widget::{AnyWidget, CommonWidget, Text, Widget, WidgetExt, WidgetId};

#[derive(Clone, Debug, Widget)]
pub struct WidgetOutline {
    #[id] id: WidgetId,
    position: Position,
    dimension: Dimension,

    state: LocalState<Box<dyn AnyWidget>>,

    child: Box<dyn AnyWidget>
}

impl WidgetOutline {
    pub fn new(state: LocalState<Box<dyn AnyWidget>>) -> WidgetOutline {
        WidgetOutline {
            id: WidgetId::new(),
            position: Default::default(),
            dimension: Default::default(),
            state: state.clone(),
            child: Text::new(Map1::read_map(state, |a| format!("{:#?}", a))).boxed(),
        }
    }
}

impl CommonWidget for WidgetOutline {
    CommonWidgetImpl!(self, position: self.position, dimension: self.dimension, child: self.child);
}