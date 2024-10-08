use std::fmt::Debug;

use carbide_macro::carbide_default_builder2;

use crate::CommonWidgetImpl;
use crate::draw::{Dimension, Position, Rect};
use crate::state::{IntoState, State};
use crate::widget::{CommonWidget, Empty, Widget, WidgetExt, WidgetId};

#[derive(Debug, Clone, Widget)]
pub struct GeometryReader<C, G> where
    C: Widget,
    G: State<T=Rect>
{
    id: WidgetId,
    child: C,
    #[state] geometry: G,
}

impl GeometryReader<Empty, Rect> {
    #[carbide_default_builder2]
    pub fn new<C: Widget, G: IntoState<Rect>>(
        geometry: G,
        child: C,
    ) -> GeometryReader<C, G::Output> {
        GeometryReader {
            id: WidgetId::new(),
            child,
            geometry: geometry.into_state(),
        }
    }
}

impl<C: Widget, G: State<T=Rect>> CommonWidget for GeometryReader<C, G> {
    fn position(&self) -> Position {
        self.geometry.value().position
    }

    fn set_position(&mut self, position: Position) {
        self.geometry.value_mut().position = position;
    }

    fn dimension(&self) -> Dimension {
        self.geometry.value().dimension
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.geometry.value_mut().dimension = dimension;
    }

    CommonWidgetImpl!(self, id: self.id, child: self.child);
}