use crate::draw::{Dimension, Position, Scalar};
use crate::flags::Flags;
use crate::focus::Focus;
use crate::layout::{BasicLayouter, Layouter};
use crate::widget::common::widget_iterator::{WidgetIter, WidgetIterMut};
use crate::widget::Id;

pub trait CommonWidget {
    fn id(&self) -> Id;
    fn set_id(&mut self, id: Id);
    fn flag(&self) -> Flags {
        Flags::EMPTY
    }

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

    fn get_focus(&self) -> Focus {
        Focus::Unfocused
    }
    fn set_focus(&mut self, focus: Focus) {}

    fn alignment(&self) -> Box<dyn Layouter> {
        Box::new(BasicLayouter::Center)
    }
    /// 0 is the most flexible and the largest number is the least flexible
    /// The flexibility of the widget determines the order of which the widgets are processed
    /// when laying out in a vertical or horizontal stack. The least flexible are processed first.
    /// If not overwritten, the default behavior is to either use the first child's flexibility or
    /// if no child are present, be 0.
    fn flexibility(&self) -> u32 {
        if let Some(first_child) = self.children().next() {
            first_child.flexibility()
        } else {
            0
        }
    }

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
    fn set_dimension(&mut self, dimension: Dimension);

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
