use uuid::Uuid;
use widget::primitive::Widget;
use ::{Point, Scalar};
use position::Dimensions;
use widget::primitive::padding::Padding;
use widget::primitive::edge_insets::EdgeInsets;
use Color;

pub trait CommonWidget {
    fn get_id(&self) -> Uuid;

    fn get_children(&self) -> &Vec<Box<dyn Widget>>;
    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Widget>>;

    fn get_position(&self) -> Point;
    fn get_x(&self) -> Scalar;
    fn set_x(&mut self, x: Scalar);

    fn get_y(&self) -> Scalar;
    fn set_y(&mut self, y: Scalar);

    fn get_size(&self) -> Dimensions;
    fn get_width(&self) -> Scalar;
    fn get_height(&self) -> Scalar;
    fn calc_width(&self, pref_width: Scalar) -> Scalar {
        self.get_width()
    }
    fn calc_height(&self, pref_height: Scalar) -> Scalar {
        self.get_height()
    }

    fn is_inside(&self, point: Point) -> bool {
        point[0] >= self.get_x()
            && point[0] < self.get_x() + self.get_width()
            && point[1] >= self.get_y()
            && point[1] < self.get_y() + self.get_height()
    }

}
