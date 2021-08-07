use uuid::Uuid;

use crate::draw::{Dimension, Position, Scalar};
use crate::flags::Flags;
use crate::widget::common::widget_iterator::{WidgetIter, WidgetIterMut};

pub trait CommonWidget {
    fn get_id(&self) -> Uuid;
    fn set_id(&mut self, id: Uuid);
    fn get_flag(&self) -> Flags;

    /// Get the logical children. This means for example for a vstack with a foreach,
    /// the children of the foreach is retrieved.
    fn get_children(&self) -> WidgetIter;
    fn get_children_mut(&mut self) -> WidgetIterMut;

    /// Get the actual children. This means for example for a vstack with a foreach,
    /// the foreach widget is retrieved.
    fn get_proxied_children(&mut self) -> WidgetIterMut;
    fn get_proxied_children_rev(&mut self) -> WidgetIterMut;

    fn get_position(&self) -> Position;
    fn set_position(&mut self, position: Position);

    fn get_x(&self) -> Scalar {
        self.get_position().x
    }

    fn set_x(&mut self, x: Scalar) {
        self.set_position(Position::new(x, self.get_y()));
    }

    fn get_y(&self) -> Scalar {
        self.get_position().y
    }

    fn set_y(&mut self, y: Scalar) {
        self.set_position(Position::new(self.get_x(), y));
    }

    fn get_dimension(&self) -> Dimension;
    fn set_dimension(&mut self, dimensions: Dimension);

    fn get_width(&self) -> Scalar {
        self.get_dimension().width
    }

    fn set_width(&mut self, width: Scalar) {
        self.set_dimension(Dimension::new(width, self.get_height()))
    }

    fn get_height(&self) -> Scalar {
        self.get_dimension().height
    }

    fn set_height(&mut self, height: Scalar) {
        self.set_dimension(Dimension::new(self.get_width(), height))
    }

    fn is_inside(&self, point: Position) -> bool {
        point.x >= self.get_x()
            && point.x < self.get_x() + self.get_width()
            && point.y >= self.get_y()
            && point.y < self.get_y() + self.get_height()
    }
}
