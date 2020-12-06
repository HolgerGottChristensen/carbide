use uuid::Uuid;
use widget::primitive::Widget;
use ::{Point, Scalar};
use position::Dimensions;
use widget::primitive::padding::Padding;
use widget::primitive::edge_insets::EdgeInsets;
use Color;
use flags::Flags;
use widget::widget_iterator::{WidgetIter, WidgetIterMut};

pub trait CommonWidget {
    fn get_id(&self) -> Uuid;
    fn get_flag(&self) -> Flags;

    fn get_children(&self) -> WidgetIter;
    fn get_children_mut(&mut self) -> WidgetIterMut;
    fn get_proxied_children(&mut self) -> WidgetIterMut;

    fn clone(&self) -> Box<Widget>;

    fn get_position(&self) -> Point;
    fn set_position(&mut self, position: Point);

    fn get_x(&self) -> Scalar {
        self.get_position()[0]
    }

    fn set_x(&mut self, x: Scalar) {
        self.set_position([x, self.get_y()]);
    }

    fn get_y(&self) -> Scalar {
        self.get_position()[1]
    }

    fn set_y(&mut self, y: Scalar) {
        self.set_position([self.get_x(), y]);
    }

    fn get_dimension(&self) -> Dimensions;
    fn set_dimension(&mut self, dimensions: Dimensions);

    fn get_width(&self) -> Scalar {
        self.get_dimension()[0]
    }

    fn set_width(&mut self, width: Scalar) {
        self.set_dimension([width, self.get_dimension()[1]])
    }

    fn get_height(&self) -> Scalar {
        self.get_dimension()[1]
    }

    fn set_height(&mut self, height: Scalar) {
        self.set_dimension([self.get_dimension()[0], height])
    }

    fn is_inside(&self, point: Point) -> bool {
        point[0] >= self.get_x()
            && point[0] < self.get_x() + self.get_width()
            && point[1] >= self.get_y()
            && point[1] < self.get_y() + self.get_height()
    }

}
