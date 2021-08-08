use crate::draw::{Dimension, Position, Scalar};
use crate::flags::Flags;
use crate::focus::Focus;
use crate::widget::common::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::widget::Id;

pub trait CommonWidget {
    fn id(&self) -> Id;
    fn set_id(&mut self, id: Id);
    fn flag(&self) -> Flags;

    /// Get the logical children. This means for example for a vstack with a foreach,
    /// the children of the foreach is retrieved.
    fn children(&self) -> WidgetIter;
    fn children_mut(&mut self) -> WidgetIterMut;

    /// Get the actual children. This means for example for a vstack with a foreach,
    /// the foreach widget is retrieved.
    fn proxied_children(&mut self) -> WidgetIterMut;
    fn proxied_children_rev(&mut self) -> WidgetIterMut;

    fn position(&self) -> Position;
    fn set_position(&mut self, position: Position);

    fn get_focus(&self) -> Focus { Focus::Unfocused }
    fn set_focus(&mut self, focus: Focus) {}

    fn x(&self) -> Scalar {
        self.position().x
    }

    fn set_x(&mut self, x: Scalar) {
        self.set_position(Position::new(x, self.y()));
    }

    fn y(&self) -> Scalar {
        self.position().y
    }

    fn set_y(&mut self, y: Scalar) {
        self.set_position(Position::new(self.x(), y));
    }

    fn dimension(&self) -> Dimension;
    fn set_dimension(&mut self, dimensions: Dimension);

    fn width(&self) -> Scalar {
        self.dimension().width
    }

    fn set_width(&mut self, width: Scalar) {
        self.set_dimension(Dimension::new(width, self.height()))
    }

    fn height(&self) -> Scalar {
        self.dimension().height
    }

    fn set_height(&mut self, height: Scalar) {
        self.set_dimension(Dimension::new(self.width(), height))
    }

    fn is_inside(&self, point: Position) -> bool {
        point.x >= self.x()
            && point.x < self.x() + self.width()
            && point.y >= self.y()
            && point.y < self.y() + self.height()
    }
}
