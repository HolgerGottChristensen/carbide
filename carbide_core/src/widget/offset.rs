use carbide_core::environment::Environment;
use carbide_macro::carbide_default_builder;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position};
use crate::layout::{BasicLayouter, Layout, Layouter};
use crate::state::{ReadState, TState};
use crate::widget::{Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(Layout)]
pub struct Offset {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    #[state]
    offset_x: TState<f64>,
    #[state]
    offset_y: TState<f64>,
}

impl Offset {
    #[carbide_default_builder]
    pub fn new(offset_x: impl Into<TState<f64>>, offset_y: impl Into<TState<f64>>, child: Box<dyn Widget>) -> Box<Self> {}

    pub fn new(offset_x: impl Into<TState<f64>>, offset_y: impl Into<TState<f64>>, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Offset {
            id: WidgetId::new(),
            child,
            position: Position::new(0.0, 0.0),
            dimension: Dimension::new(0.0, 0.0),
            offset_x: offset_x.into(),
            offset_y: offset_y.into(),
        })
    }
}

impl Layout for Offset {
    fn position_children(&mut self, env: &mut Environment) {
        let positioning = BasicLayouter::Center.positioner();
        let position = self.position;
        let dimension = self.dimension;

        positioning(position, dimension, &mut self.child);

        let mut child_position = self.child.position();

        child_position.x += *self.offset_x.value();
        child_position.y += *self.offset_y.value();

        self.child.set_position(child_position);

        self.child.position_children(env);
    }
}

CommonWidgetImpl!(Offset, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl WidgetExt for Offset {}
