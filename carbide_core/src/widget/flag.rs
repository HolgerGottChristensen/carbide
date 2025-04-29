use crate::state::ReadState;
use crate::widget::Identifiable;
use crate::widget::CommonWidget;

use crate::draw::{Dimension, Position};
use crate::misc::flags::WidgetFlag;
use crate::state::IntoReadState;
use crate::widget::{Empty, IntoWidget, Widget, WidgetId};
use crate::CommonWidgetImpl;

#[derive(Debug, Clone, Widget)]
pub struct Flagged<C, F> where C: Widget, F: ReadState<T=WidgetFlag> {
    child: C,
    flags: F,
}

impl Flagged<Empty, WidgetFlag> {
    pub fn new<C: IntoWidget, F: IntoReadState<WidgetFlag>>(child: C, flags: F) -> Flagged<C::Output, F::Output> {
        Flagged {
            child: child.into_widget(),
            flags: flags.into_read_state(),
        }
    }
}

impl<C: Widget, F: ReadState<T=WidgetFlag>> Identifiable for Flagged<C, F> {
    fn id(&self) -> WidgetId {
        self.child.id()
    }
}

impl<C: Widget, F: ReadState<T=WidgetFlag>> CommonWidget for Flagged<C, F> {
    CommonWidgetImpl!(self, child: self.child);

    fn position(&self) -> Position {
        self.child.position()
    }

    fn set_position(&mut self, position: Position) {
        self.child.set_position(position)
    }

    fn dimension(&self) -> Dimension {
        self.child.dimension()
    }

    fn set_dimension(&mut self, dimension: Dimension) {
        self.child.set_dimension(dimension)
    }

    fn flag(&self) -> WidgetFlag {
        *self.flags.value()
    }
}